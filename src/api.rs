// Access items defined in the crate root (main.rs)
use super::{AppState, CommentRecord};
use crate::scrape::get_comments;
use crate::utils::create_batches;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::{error, info};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
pub(crate) struct ScrapeRequest {
    pub url: String,
}

/// Health check handler: echoes back the provided `msg` with current Unix timestamp
#[axum::debug_handler]
pub async fn ping(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    match params.get("msg").filter(|m| !m.is_empty()) {
        Some(msg) => {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let body = format!("{} {}", msg, ts);
            (StatusCode::OK, body)
        }
        None => (
            StatusCode::BAD_REQUEST,
            "missing required query parameter: msg".to_string(),
        ),
    }
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

    info!("/scrape called; starting scraping for {}", target_url);
    let comments_retrieval = get_comments(&target_url).await;
    match comments_retrieval {
        Ok(comments) => {
            info!("Parsed {} root comments", comments.len());

            let batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);
            let mut total_inserted = 0usize;
            for batch in batches.iter() {
                match insert_comments(&state.db_pool, batch).await {
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

async fn insert_comments(
    pool: &Pool<Sqlite>,
    comments: &Vec<CommentRecord>,
) -> Result<usize, sqlx::Error> {
    let mut inserted = 0usize;
    for comment in comments {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO comments (id, author, date, text) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(comment.id)
        .bind(&comment.author)
        .bind(&comment.date)
        .bind(&comment.text)
        .execute(pool)
        .await?;
        inserted += result.rows_affected() as usize; // OR IGNORE returns 0 when skipped due to PK conflict
    }
    Ok(inserted)
}
