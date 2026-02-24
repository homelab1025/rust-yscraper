use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::env;
//
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use testcontainers_modules::postgres::Postgres;
use web_server::CommentRecord;
use web_server::db::comments_repository::CommentsRepository;
use web_server::db::links_repository::LinksRepository;
use web_server::db::postgresql::PgCommentsRepository;

async fn setup_db() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .with_user("postgres")
        .with_password("postgres")
        .with_db_name("postgres")
        .with_network("bridge")
        .with_container_name("postgres_db_test")
        .with_log_consumer(|log: &testcontainers::core::logs::LogFrame| {
            print!("{}", String::from_utf8_lossy(log.bytes()));
        })
        .start()
        .await
        .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    // Get the internal bridge IP of the Postgres container for Liquibase to connect to.
    let postgres_ip = container.get_bridge_ip_address().await.unwrap();
    let jdbc_url = format!("jdbc:postgresql://{}:5432/postgres", postgres_ip);

    let project_root = env::current_dir().unwrap().parent().unwrap().to_path_buf();
    let db_path = project_root.join("db");
    let db_path_str = db_path.to_str().unwrap();

    let liquibase = GenericImage::new("liquibase/liquibase", "4.23")
        .with_network("bridge")
        .with_mount(testcontainers::core::Mount::bind_mount(
            db_path_str,
            "/liquibase/db",
        ))
        .with_log_consumer(|log: &testcontainers::core::logs::LogFrame| {
            print!("{}", String::from_utf8_lossy(log.bytes()));
        })
        .with_cmd([
            "--changelog-file=db/changelog/db.changelog-master.yaml",
            &format!("--url={}", jdbc_url),
            "--username=postgres",
            "--password=postgres",
            "update",
        ])
        .with_ready_conditions(vec![WaitFor::message_on_stdout("UPDATE SUMMARY")]);

    let _liquibase_container = AsyncRunner::start(liquibase).await.unwrap();

    let conn_str = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
    let pool = PgPool::connect(&conn_str)
        .await
        .expect("Failed to connect to Postgres");

    (pool, container)
}

#[tokio::test]
async fn test_get_urls_due_for_refresh() {
    let (pool, _container) = setup_db().await;
    let repo = PgCommentsRepository::new(pool.clone());

    let now = Utc::now();

    // 1. Due: last_scraped is NULL, within days_limit
    let id1 = 1;
    let url1 = "http://example.com/1";
    sqlx::query("INSERT INTO urls (id, url, date_added, last_scraped, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id1)
        .bind(url1)
        .bind(now - Duration::days(2))
        .bind(None::<chrono::DateTime<Utc>>)
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    // 2. Not Due: last_scraped is recent (1 hour ago)
    let id2 = 2;
    let url2 = "http://example.com/2";
    sqlx::query("INSERT INTO urls (id, url, date_added, last_scraped, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id2)
        .bind(url2)
        .bind(now - Duration::days(2))
        .bind(Some(now - Duration::hours(1)))
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    // 3. Due: last_scraped is old (25 hours ago)
    let id3 = 3;
    let url3 = "http://example.com/3";
    sqlx::query("INSERT INTO urls (id, url, date_added, last_scraped, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id3)
        .bind(url3)
        .bind(now - Duration::days(2))
        .bind(Some(now - Duration::hours(25)))
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    // 4. Not Due: expired (added 10 days ago, limit 7)
    let id4 = 4;
    let url4 = "http://example.com/4";
    sqlx::query("INSERT INTO urls (id, url, date_added, last_scraped, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id4)
        .bind(url4)
        .bind(now - Duration::days(10))
        .bind(Some(now - Duration::hours(25)))
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    let due_urls = repo.get_urls_due_for_refresh().await.unwrap();

    assert_eq!(due_urls.len(), 2, "Should have 2 urls due for refresh");

    // Check Case 1 (last_scraped NULL should be first)
    assert_eq!(due_urls[0].id, id1);
    assert_eq!(due_urls[0].url, url1);
    assert!(due_urls[0].last_scraped.is_none());

    // Check Case 3
    assert_eq!(due_urls[1].id, id3);
    assert_eq!(due_urls[1].url, url3);
    assert!(due_urls[1].last_scraped.is_some());
}

#[tokio::test]
async fn test_delete_link_with_comments() {
    let (pool, _container) = setup_db().await;
    let repo = PgCommentsRepository::new(pool.clone());

    let now = Utc::now();
    let id = 100;
    let url = "http://example.com/delete_test";

    // 1. Insert a link
    sqlx::query("INSERT INTO urls (id, url, date_added, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(url)
        .bind(now)
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    // 2. Insert comments for that link
    sqlx::query(
        "INSERT INTO comments (id, author, date, text, url_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(1)
    .bind("author1")
    .bind("2026-01-01T00:00:00Z")
    .bind("comment 1")
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO comments (id, author, date, text, url_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(2)
    .bind("author2")
    .bind("2026-01-02T00:00:00Z")
    .bind("comment 2")
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    // 3. Verify they exist
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE url_id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);

    // 4. Delete the link
    let affected = repo.delete_link(id).await.unwrap();
    assert_eq!(affected, 1);

    // 5. Verify the link is gone
    let link_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM urls WHERE id = $1)")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!link_exists);

    // 6. Verify comments are gone
    let comments_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE url_id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(comments_count, 0);
}

#[tokio::test]
async fn test_upsert_comments_selective_update() {
    let (pool, _container) = setup_db().await;
    let repo = PgCommentsRepository::new(pool.clone());

    let url_id = 200;
    let comment_id = 500;

    // 1. Setup a link
    sqlx::query("INSERT INTO urls (id, url, date_added, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5)")
        .bind(url_id)
        .bind("http://example.com/upsert_test")
        .bind(Utc::now())
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    // 2. Initial insert via upsert_comments
    let initial_comments = vec![CommentRecord {
        id: comment_id,
        author: "original_author".to_string(),
        date: "2026-01-01".to_string(),
        text: "original_text".to_string(),
        tags: vec![],
        state: web_server::CommentState::Picked, // state = 1
    }];

    repo.upsert_comments(&initial_comments, url_id)
        .await
        .unwrap();

    // 3. Verify initial state
    let row: (String, String, i32) =
        sqlx::query_as("SELECT author, text, state FROM comments WHERE id = $1")
            .bind(comment_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(row.0, "original_author");
    assert_eq!(row.1, "original_text");
    assert_eq!(row.2, 1);

    // 4. Upsert with NEW text but different fields (author, date, state)
    // The requirement is that ONLY text should be updated.
    let updated_comments = vec![CommentRecord {
        id: comment_id,
        author: "NEW_author_should_be_ignored".to_string(),
        date: "2026-99-99".to_string(),
        text: "UPDATED_text".to_string(),
        tags: vec!["ignored_tag".to_string()],
        state: web_server::CommentState::Discarded, // state = 2, should be ignored
    }];

    repo.upsert_comments(&updated_comments, url_id)
        .await
        .unwrap();

    // 5. Verify that ONLY text changed
    let row: (String, String, i32) =
        sqlx::query_as("SELECT author, text, state FROM comments WHERE id = $1")
            .bind(comment_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(
        row.0, "original_author",
        "Author should not have been updated"
    );
    assert_eq!(row.1, "UPDATED_text", "Text should have been updated");
    assert_eq!(row.2, 1, "State should not have been updated");
}
