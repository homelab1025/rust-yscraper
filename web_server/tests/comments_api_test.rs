mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;
use web_server::CommentState;
use web_server::api::comments::{CommentDto, CommentsPage};
use web_server::api::links::LinkDto;

async fn insert_url(pool: &PgPool, id: i64) {
    sqlx::query("INSERT INTO urls (id, url, date_added, frequency_hours, days_limit) VALUES ($1, $2, NOW(), $3, $4)")
        .bind(id)
        .bind(format!("http://example.com/{}", id))
        .bind(24)
        .bind(7)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_comment(pool: &PgPool, id: i64, url_id: i64, state: i32) {
    sqlx::query("INSERT INTO comments (id, author, date, text, url_id, state) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(format!("user{}", id))
        .bind("2026-01-01T00:00:00Z")
        .bind(format!("comment text {}", id))
        .bind(url_id)
        .bind(state)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_list_comments_returns_data() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 2).await;
    insert_comment(&pool, 10, 2, 0).await;
    insert_comment(&pool, 11, 2, 1).await;

    let app = web_server::build_router(common::make_test_app_state(pool));
    let req = Request::builder()
        .method("GET")
        .uri("/comments?url_id=2")
        .body(Body::empty())
        .unwrap();

    let resp = ServiceExt::<Request<Body>>::oneshot(app, req)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let page: CommentsPage = serde_json::from_slice(&body).unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 2);
    // Verify field mapping: DB column `author` is exposed as `user` in the DTO
    let item = &page.items[0];
    assert!(!item.user.is_empty());
    assert!(!item.text.is_empty());
}

#[tokio::test]
async fn test_list_comments_state_filter() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 3).await;
    insert_comment(&pool, 20, 3, 0).await; // New
    insert_comment(&pool, 21, 3, 1).await; // Picked
    insert_comment(&pool, 22, 3, 1).await; // Picked

    let app = web_server::build_router(common::make_test_app_state(pool));
    let req = Request::builder()
        .method("GET")
        .uri("/comments?url_id=3&state=PICKED")
        .body(Body::empty())
        .unwrap();

    let resp = ServiceExt::<Request<Body>>::oneshot(app, req)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let page: CommentsPage = serde_json::from_slice(&body).unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn test_list_comments_pagination() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 4).await;
    for i in 30..35i64 {
        insert_comment(&pool, i, 4, 0).await;
    }

    let app = web_server::build_router(common::make_test_app_state(pool));
    let req = Request::builder()
        .method("GET")
        .uri("/comments?url_id=4&offset=1&count=2")
        .body(Body::empty())
        .unwrap();

    let resp = ServiceExt::<Request<Body>>::oneshot(app, req)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let page: CommentsPage = serde_json::from_slice(&body).unwrap();
    assert_eq!(page.total, 5);
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn test_update_comment_state_picked() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 5).await;
    insert_comment(&pool, 40, 5, 0).await;

    let app = web_server::build_router(common::make_test_app_state(pool));

    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/comments/40/state")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"state": "PICKED"}"#))
        .unwrap();
    let resp = app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let get_req = Request::builder()
        .method("GET")
        .uri("/comments/40")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(get_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let comment: CommentDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(comment.state, CommentState::Picked);
}

#[tokio::test]
async fn test_update_comment_state_discarded() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 6).await;
    insert_comment(&pool, 41, 6, 0).await;

    let app = web_server::build_router(common::make_test_app_state(pool));

    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/comments/41/state")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"state": "DISCARDED"}"#))
        .unwrap();
    let resp = app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let get_req = Request::builder()
        .method("GET")
        .uri("/comments/41")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(get_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let comment: CommentDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(comment.state, CommentState::Discarded);
}

#[tokio::test]
async fn test_update_comment_state_discarded_updates_url_count() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 7).await;
    insert_comment(&pool, 50, 7, 0).await;

    let app = web_server::build_router(common::make_test_app_state(pool));

    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/comments/50/state")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"state": "DISCARDED"}"#))
        .unwrap();
    let resp = app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let list_req = Request::builder()
        .method("GET")
        .uri("/links")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let links: Vec<LinkDto> = serde_json::from_slice(&body).unwrap();
    let link = links.iter().find(|l| l.id == 7).unwrap();
    assert_eq!(link.discarded_comment_count, 1);
}
