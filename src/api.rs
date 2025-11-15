// Access items defined in the crate root (main.rs)
use crate::scrape::get_comments as scrape_get_comments;
use crate::utils::{create_batches, extract_item_id_from_url};
use crate::{AppState, CommentRecord};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
pub struct ScrapeRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub msg: String,
}

/// Health check handler: echoes back the provided `msg` with the current Unix timestamp
#[axum::debug_handler]
pub async fn ping(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PingResponse>, (StatusCode, String)> {
    match params.get("msg").filter(|m| !m.is_empty()) {
        Some(msg) => {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let body = format!("{} {}", msg, ts);
            Ok(Json(PingResponse { msg: body }))
        }
        None => Err((
            StatusCode::BAD_REQUEST,
            "missing required query parameter: msg".to_string(),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentsQuery {
    pub offset: Option<i64>,
    pub count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CommentDto {
    pub id: i64,
    pub text: String,
    pub user: String,
    pub url_id: i64,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct CommentsPage {
    pub total: i64,
    pub items: Vec<CommentDto>,
}

/// GET /comments — returns comments ordered by date desc with pagination
#[axum::debug_handler]
pub async fn list_comments(
    State(state): State<AppState>,
    Query(filter): Query<CommentsQuery>,
) -> impl IntoResponse {
    let offset = filter.offset.unwrap_or(0).max(0);
    let count = filter.count.unwrap_or(10).clamp(1, 100);

    // total count
    let total = match state.repo.count_comments().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to count comments: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch total").into_response();
        }
    };

    // page items ordered by date desc; fallback id desc for ties
    let rows = match state.repo.page_comments(offset, count).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch comments page: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to fetch comments",
            )
                .into_response();
        }
    };

    let items: Vec<CommentDto> = rows
        .into_iter()
        .map(|row| CommentDto {
            id: row.id,
            user: row.author,
            date: row.date,
            text: row.text,
            url_id: row.url_id,
        })
        .collect();

    let body = CommentsPage { total, items };
    Json(body).into_response()
}

/// Triggers scraping and inserts results into the database.
#[axum::debug_handler]
pub async fn scrape_hackernews(
    State(state): State<AppState>,
    Json(payload): Json<ScrapeRequest>,
) -> impl IntoResponse {
    let target_url = payload.url.trim().to_string();

    // Validate URL form: must start with the HN item base URL
    let res = validate_url(&target_url);
    if let Err(e) = res {
        return (StatusCode::BAD_REQUEST, e);
    }

    // Extract the HN item id from the URL
    let url_id = match extract_item_id_from_url(&target_url) {
        Some(id) => id,
        None => {
            error!("Unable to extract id= from url: {}", &target_url);
            return (
                StatusCode::BAD_REQUEST,
                "invalid Hacker News item url; missing id query parameter".to_string(),
            );
        }
    };

    // Ensure the URL is recorded in the urls table and get (or confirm) its id
    if let Err(e) = state.repo.upsert_url(url_id, &target_url).await {
        error!("Failed to upsert url: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record url".to_string(),
        );
    }

    info!("/scrape called; starting scraping for {}", target_url);
    let comments_retrieval = scrape_get_comments(&target_url).await;
    match comments_retrieval {
        Ok(comments) => {
            info!("Parsed {} root comments", comments.len());

            let batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);
            let mut total_inserted = 0usize;
            for batch in batches.iter() {
                match state.repo.upsert_comments(batch, url_id).await {
                    Ok(n) => {
                        total_inserted += n;
                        info!("Inserted {} comments into the database", n);
                    }
                    Err(e) => error!("Failed to insert comments: {}", e),
                }
            }

            (
                StatusCode::OK,
                format!("ok: parsed={}, inserted={}", comments.len(), total_inserted),
            )
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to scrape: {}", error),
        ),
    }
}

fn validate_url(target_url: &String) -> Result<(), String> {
    let required_prefix = "https://news.ycombinator.com/item";
    if !target_url.starts_with(required_prefix) {
        error!("/scrape invalid url provided: {}", target_url);
        return Err(format!("/scrape invalid url provided: {}", target_url));
    }

    Ok(())
}
