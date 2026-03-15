mod common;

use chrono::Utc;
use web_server::db::comments_repository::CommentsRepository;
use web_server::db::postgresql::PgCommentsRepository;

#[tokio::test]
async fn test_page_comments_sorting() {
    let (pool, _container) = common::setup_db().await;
    let repo = PgCommentsRepository::new(pool.clone());

    let url_id = 1;
    sqlx::query("INSERT INTO urls (id, url, date_added, frequency_hours, days_limit) VALUES ($1, $2, $3, $4, $5)")
        .bind(url_id)
        .bind("http://example.com")
        .bind(Utc::now())
        .bind(24)
        .bind(7)
        .execute(&pool)
        .await
        .unwrap();

    // Insert 3 comments with different dates and subcomment counts
    // Comment 1: Oldest, Most subcomments
    sqlx::query("INSERT INTO comments (id, author, date, text, url_id, subcomment_count) VALUES (1, 'user1', '2026-01-01T10:00:00Z', 'text1', 1, 10)")
        .execute(&pool).await.unwrap();
    // Comment 2: Middle date, Least subcomments
    sqlx::query("INSERT INTO comments (id, author, date, text, url_id, subcomment_count) VALUES (2, 'user2', '2026-01-02T10:00:00Z', 'text2', 1, 0)")
        .execute(&pool).await.unwrap();
    // Comment 3: Newest, Middle subcomments
    sqlx::query("INSERT INTO comments (id, author, date, text, url_id, subcomment_count) VALUES (3, 'user3', '2026-01-03T10:00:00Z', 'text3', 1, 5)")
        .execute(&pool).await.unwrap();

    // 1. Sort by Date DESC (Default)
    let rows = repo.page_comments(0, 10, url_id, None, Some(web_server::SortBy::Date), Some(web_server::SortOrder::Desc)).await.unwrap();
    assert_eq!(rows[0].id, 3);
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[2].id, 1);

    // 2. Sort by Date ASC
    let rows = repo.page_comments(0, 10, url_id, None, Some(web_server::SortBy::Date), Some(web_server::SortOrder::Asc)).await.unwrap();
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[2].id, 3);

    // 3. Sort by Subcomment Count DESC
    let rows = repo.page_comments(0, 10, url_id, None, Some(web_server::SortBy::SubcommentCount), Some(web_server::SortOrder::Desc)).await.unwrap();
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[1].id, 3);
    assert_eq!(rows[2].id, 2);

    // 4. Sort by Subcomment Count ASC
    let rows = repo.page_comments(0, 10, url_id, None, Some(web_server::SortBy::SubcommentCount), Some(web_server::SortOrder::Asc)).await.unwrap();
    assert_eq!(rows[0].id, 2);
    assert_eq!(rows[1].id, 3);
    assert_eq!(rows[2].id, 1);
}
