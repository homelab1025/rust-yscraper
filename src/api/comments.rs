use super::common::{ApiError, ApiErrorCode};
use crate::api::app_state::CommentsAppState;
use crate::api::scrape_task::ScrapeTask;
use crate::utils::extract_item_id_from_url;
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use log::error;
use serde::{Deserialize, Serialize};

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

    let scrape_task = ScrapeTask::new(target_url, url_id, state.repo.clone());

    let _schedule_res = state.task_queue.schedule(scrape_task).await;

    // TODO: return a 202 Accepted response and refactor this to Json structure
    (StatusCode::ACCEPTED, "ok".to_string())
}

fn validate_url(target_url: &String) -> Result<(), String> {
    let required_prefix = "https://news.ycombinator.com/item";
    if !target_url.starts_with(required_prefix) {
        error!("/scrape invalid url provided: {}", target_url);
        return Err(format!("/scrape invalid url provided: {}", target_url));
    }

    Ok(())
}
