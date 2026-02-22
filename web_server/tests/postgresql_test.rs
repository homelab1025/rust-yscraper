use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::env;
//
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use testcontainers_modules::postgres::Postgres;
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

    let _liquibase_container = AsyncRunner::start(liquibase)
        .await
        .unwrap();

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
