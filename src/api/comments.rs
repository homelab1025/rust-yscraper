use super::common::{ApiError, ApiErrorCode};
use crate::api::app_state::CommentsAppState;
use crate::api::scrape_task::ScrapeTask;
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    pub item_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ScrapeState {
    Scheduled,
    AlreadyScheduled,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeResponse {
    pub state: ScrapeState,
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
) -> Result<Json<ScrapeResponse>, (StatusCode, Json<ApiError>)> {

    let item_id = payload.item_id;
    let target_url = format!("https://news.ycombinator.com/item?id={}", item_id);

    let scrape_task = ScrapeTask::new(target_url, item_id, state.repo.clone());
    let schedule_res = state.task_queue.schedule(scrape_task).await;

    match schedule_res {
        Ok(true) => {
            info!("Scraping task scheduled successfully.");
            Ok(Json(ScrapeResponse {
                state: ScrapeState::Scheduled,
            }))
        }
        Ok(false) => {
            info!("Scraping task already scheduled.");
            Ok(Json(ScrapeResponse {
                state: ScrapeState::AlreadyScheduled,
            }))
        }
        Err(e) => {
            error!("Failed to schedule scrape task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::SchedulingError,
                    msg: String::from("could not schedule scrape task"),
                }),
            ))
        }
    }
}
