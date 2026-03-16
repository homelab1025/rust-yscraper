mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use web_server::api::links::{LinkDto, ScrapeResponse, ScrapeState};

async fn insert_url(pool: &PgPool, id: i64, thread_month: Option<i32>, thread_year: Option<i32>) {
    sqlx::query("INSERT INTO urls (id, url, date_added, frequency_hours, days_limit, thread_month, thread_year) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(id)
        .bind(format!("http://example.com/{}", id))
        .bind(Utc::now())
        .bind(24)
        .bind(7)
        .bind(thread_month)
        .bind(thread_year)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_list_links_returns_data() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 1, Some(3), Some(2026)).await;

    let app = web_server::build_router(common::make_test_app_state(pool));
    let req = Request::builder()
        .method("GET")
        .uri("/links")
        .body(Body::empty())
        .unwrap();

    let resp = ServiceExt::<Request<Body>>::oneshot(app, req)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let links: Vec<LinkDto> = serde_json::from_slice(&body).unwrap();
    assert_eq!(links.len(), 1);
    let link = &links[0];
    assert_eq!(link.id, 1);
    assert_eq!(link.total_comment_count, 0);
    assert_eq!(link.picked_comment_count, 0);
    assert_eq!(link.discarded_comment_count, 0);
    assert_eq!(link.thread_month, Some(3));
    assert_eq!(link.thread_year, Some(2026));
}

#[tokio::test]
async fn test_delete_link_returns_200() {
    let (pool, _container) = common::setup_db().await;
    insert_url(&pool, 1, None, None).await;

    let app = web_server::build_router(common::make_test_app_state(pool));

    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/links/1")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(delete_req).await.unwrap();
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
    assert!(links.is_empty());
}

#[tokio::test]
async fn test_delete_link_returns_404() {
    let (pool, _container) = common::setup_db().await;

    let app = web_server::build_router(common::make_test_app_state(pool));
    let req = Request::builder()
        .method("DELETE")
        .uri("/links/999")
        .body(Body::empty())
        .unwrap();

    let resp = ServiceExt::<Request<Body>>::oneshot(app, req)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_scrape_link_schedules_task() {
    let (pool, _container) = common::setup_db().await;
    let scheduled = Arc::new(Mutex::new(vec![]));
    let scheduler = Arc::new(common::RecordingScheduler { scheduled: scheduled.clone(), outcome: true });
    let app = web_server::build_router(common::make_test_app_state_with_scheduler(pool, scheduler));

    let req = Request::builder()
        .method("POST")
        .uri("/scrape")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"item_id": 12345}"#))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let scrape_resp: ScrapeResponse = serde_json::from_slice(&body).unwrap();
    assert!(matches!(scrape_resp.state, ScrapeState::Scheduled));

    let tasks = scheduled.lock().await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].0, 12345);
    assert_eq!(tasks[0].1, "https://news.ycombinator.com/item?id=12345");
    drop(tasks);

    let list_req = Request::builder()
        .method("GET")
        .uri("/links")
        .body(Body::empty())
        .unwrap();
    let list_resp = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(list_resp.into_body(), usize::MAX).await.unwrap();
    let links: Vec<LinkDto> = serde_json::from_slice(&body).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].id, 12345);
}

#[tokio::test]
async fn test_scrape_link_already_scheduled() {
    let (pool, _container) = common::setup_db().await;
    let scheduled = Arc::new(Mutex::new(vec![]));
    let scheduler = Arc::new(common::RecordingScheduler { scheduled: scheduled.clone(), outcome: false });
    let app = web_server::build_router(common::make_test_app_state_with_scheduler(pool, scheduler));

    let req = Request::builder()
        .method("POST")
        .uri("/scrape")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"item_id": 99}"#))
        .unwrap();

    let resp = ServiceExt::<Request<Body>>::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let scrape_resp: ScrapeResponse = serde_json::from_slice(&body).unwrap();
    assert!(matches!(scrape_resp.state, ScrapeState::AlreadyScheduled));
}
