use crate::utils::{create_batches, extract_item_id_from_url};
use crate::{CommentRecord, CommentsAppState};
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use log::{error, info};
use serde::{Deserialize, Serialize};
use crate::scrape::scrape::{get_comments, ScrapeTask};
use super::common::{ApiError, ApiErrorCode};

#[derive(Debug, Deserialize)]
pub struct CommentsQuery {
    pub offset: Option<i64>,
    pub count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentDto {
    pub id: i64,
    pub text: String,
    pub user: String,
    pub url_id: i64,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentsPage {
    pub total: i64,
    pub items: Vec<CommentDto>,
}

#[derive(Debug, Deserialize)]
pub struct ScrapeRequest {
    pub url: String,
}

/// GET /commentsurl_idurns comments ordered by date desc with pagination
#[axum::debug_handler]
pub async fn list_comments(
    State(state): State<CommentsAppState>,
    Query(filter): Query<CommentsQuery>,
) -> Result<Json<CommentsPage>, (StatusCode, Json<ApiError>)> {
    let offset = filter.offset.unwrap_or(0).max(0);
    let count = filter.count.unwrap_or(10).clamp(1, 100);

    // total count
    let total = match state.repo.count_comments().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to count comments: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::DatabaseError,
                    msg: "failed to count comments".to_string(),
                }),
            ))
        }?,
    };

    // page items ordered by date desc; fallback id desc for ties
    let rows = match state.repo.page_comments(offset, count).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch comments page: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::DatabaseError,
                    msg: "failed to count comments".to_string(),
                }),
            ))
        }?,
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
    Ok(Json(body))
}

/// Triggers scraping and inserts results into the database.
#[axum::debug_handler]
pub async fn scrape_comments(
    State(state): State<CommentsAppState>,
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

    let scrape_task = ScrapeTask {
        url: target_url.clone(),
        url_id: url_id
    };

    let _schedule_res = state.task_queue.schedule(scrape_task).await;

    // Ensure the URL is recorded in the urls table and get (or confirm) its id
    if let Err(e) = state.repo.upsert_url(url_id, &target_url).await {
        error!("Failed to upsert url: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record url".to_string(),
        );
    }

    info!("/scrape called; starting scraping for {}", target_url);
    let comments_retrieval = get_comments(&target_url).await;
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
