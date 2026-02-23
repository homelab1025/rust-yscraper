use crate::api::app_state::AppState;
use crate::api::common::{ApiError, ApiErrorCode};
use crate::db::CombinedRepository;
use crate::scrape_task::ScrapeTask;
use crate::task_queue::TaskScheduler;
use axum::Json;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone)]
pub struct LinksAppState {
    pub repo: Arc<dyn CombinedRepository>,
    pub task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    pub default_days_limit: u32,
    pub default_frequency_hours: u32,
}

impl FromRef<AppState> for LinksAppState {
    fn from_ref(input: &AppState) -> Self {
        LinksAppState {
            repo: input.repo.clone(),
            task_queue: input.task_queue.clone(),
            default_days_limit: input.config.default_days_limit,
            default_frequency_hours: input.config.default_frequency_hours,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq)]
pub struct LinkDto {
    pub id: i64,
    pub url: String,
    pub date_added: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ScrapeRequest {
    pub item_id: i64,
    /// Number of days to keep refreshing comments (default: 7)
    pub days_limit: Option<u32>,
    /// Frequency in hours between refreshes (default: 24)
    pub frequency_hours: Option<u32>,
}

fn validate_scrape_request(request: &ScrapeRequest, default_days_limit: u32, default_frequency_hours: u32) -> Result<(u32, u32), ApiError> {
    let days_limit = request.days_limit.unwrap_or(default_days_limit);
    let frequency_hours = request.frequency_hours.unwrap_or(default_frequency_hours);
    
    if days_limit == 0 {
        return Err(ApiError {
            code: ApiErrorCode::BadRequest,
            msg: "days_limit must be greater than 0".to_string(),
        });
    }
    
    if frequency_hours == 0 {
        return Err(ApiError {
            code: ApiErrorCode::BadRequest,
            msg: "frequency_hours must be greater than 0".to_string(),
        });
    }
    
    Ok((days_limit, frequency_hours))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum ScrapeState {
    Scheduled,
    AlreadyScheduled,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScrapeResponse {
    pub state: ScrapeState,
}

/// Triggers scraping and inserts results into the database.
/// Trigger a scrape task for a specific Hacker News item
#[utoipa::path(
    post,
    path = "/scrape",
    request_body = ScrapeRequest,
    responses(
        (status = 200, description = "Scrape task scheduled or already scheduled", body = ScrapeResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
#[axum::debug_handler]
pub async fn scrape_link(
    State(state): State<LinksAppState>,
    Json(payload): Json<ScrapeRequest>,
) -> Result<Json<ScrapeResponse>, (StatusCode, Json<ApiError>)> {
    // Validate request and extract defaults
    let (days_limit, frequency_hours) = validate_scrape_request(&payload, state.default_days_limit, state.default_frequency_hours)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    let item_id = payload.item_id;
    let target_url = format!("https://news.ycombinator.com/item?id={}", item_id);

    // Store URL with scheduling metadata (always succeeds)
    if let Err(e) = state.repo.upsert_url_with_scheduling(
        item_id, 
        &target_url, 
        frequency_hours, 
        days_limit
    ).await {
        error!("Failed to upsert URL with scheduling: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: ApiErrorCode::DatabaseError,
                msg: "failed to schedule URL".to_string(),
            }),
        ));
    }

    // REFACTOR: we need to rethink what gets passed to the scrape task and unify this with the background scheduler.
    // Always schedule the initial scrape
    let scrape_task = ScrapeTask::new(target_url, item_id, state.repo.clone());
    let schedule_res = state.task_queue.schedule(scrape_task).await;

    match schedule_res {
        Ok(true) => {
            info!("Scraping task scheduled successfully with {}-day limit and {}-hour frequency.", 
                  days_limit, frequency_hours);
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
                    msg: "could not schedule scrape task".to_string(),
                }),
            ))
        }
    }
}

/// Retrieve all links with their item IDs and added date.
#[utoipa::path(
    get,
    path = "/links",
    responses(
        (status = 200, description = "List of all links", body = [LinkDto]),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn list_links(
    State(state): State<LinksAppState>,
) -> Result<Json<Vec<LinkDto>>, (StatusCode, Json<ApiError>)> {
    let links = state.repo.list_links().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: ApiErrorCode::DatabaseError,
                msg: format!("Database error: {}", e),
            }),
        )
    })?;

    let dtos = links
        .into_iter()
        .map(|row| LinkDto {
            id: row.id,
            url: row.url,
            date_added: row.date_added.to_rfc3339(),
        })
        .collect();

    Ok(Json(dtos))
}

#[utoipa::path(
    delete,
    path = "/links/{id}",
    params(
        ("id" = i64, Path, description = "Link ID to delete")
    ),
    responses(
        (status = 200, description = "Link deleted successfully"),
        (status = 404, description = "Link not found", body = ApiError)
    )
)]
pub async fn delete_link(
    State(state): State<LinksAppState>,
    Path(id): Path<i64>,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    match state.repo.delete_link(id).await {
        Ok(n) => {
            if n == 0 {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiError {
                        code: ApiErrorCode::NotFound,
                        msg: format!("Link with ID {} not found", id),
                    }),
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            log::error!("Failed to delete link: {}", e);

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::DatabaseError,
                    msg: format!("Failed to delete link: {}", e),
                }),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use async_trait::async_trait;
    use chrono::Utc;
    use sqlx::Error;
    use tokio::sync::mpsc::error::TrySendError;

    struct MockRepo {
        links: Result<Vec<DbUrlRow>, String>,
        delete_result: Result<u64, String>,
    }

    #[async_trait]
    impl CommentsRepository for MockRepo {
        async fn count_comments(&self, _url_id: Option<i64>) -> Result<i64, sqlx::Error> { Ok(0) }
        async fn page_comments(&self, _offset: i64, _count: i64, _url_id: Option<i64>) -> Result<Vec<DbCommentRow>, sqlx::Error> { Ok(vec![]) }
        async fn upsert_comments(&self, _comments: &[crate::CommentRecord], _url_id: i64) -> Result<usize, sqlx::Error> { Ok(0) }
    }

    #[async_trait]
    impl LinksRepository for MockRepo {
        async fn list_links(&self) -> Result<Vec<DbUrlRow>, Error> {
            self.links.clone().map_err(Error::Protocol)
        }
        async fn delete_link(&self, _id: i64) -> Result<u64, sqlx::Error> {
            self.delete_result.clone().map_err(Error::Protocol)
        }
        async fn upsert_url_with_scheduling(&self, _id: i64, _url: &str, _frequency_hours: u32, _days_limit: u32) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(vec![])
        }
        async fn update_last_scraped(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum ScheduleOutcome {
        Scheduled,
        AlreadyInQueue,
    }

    struct StubScheduler {
        outcome: ScheduleOutcome,
    }

    #[async_trait]
    impl TaskScheduler<ScrapeTask> for StubScheduler {
        async fn schedule(&self, _task: ScrapeTask) -> Result<bool, TrySendError<ScrapeTask>> {
            match self.outcome {
                ScheduleOutcome::Scheduled => Ok(true),
                ScheduleOutcome::AlreadyInQueue => Ok(false),
            }
        }
    }

    fn make_state(
        links: Result<Vec<DbUrlRow>, String>,
        delete_result: Result<u64, String>,
        schedule_outcome: ScheduleOutcome,
    ) -> LinksAppState {
        LinksAppState {
            repo: Arc::new(MockRepo { links, delete_result }),
            task_queue: Arc::new(StubScheduler { outcome: schedule_outcome }),
            default_days_limit: 7,
            default_frequency_hours: 24,
        }
    }

    #[tokio::test]
    async fn test_list_links_success() {
        let time_url1 = Utc::now();
        let time_url2 = Utc::now();
        let rows = vec![
            DbUrlRow { id: 1, url: "https://example.com/1".to_string(), date_added: time_url1 },
            DbUrlRow { id: 2, url: "https://example.com/2".to_string(), date_added: time_url2 },
        ];
        let state = make_state(Ok(rows.clone()), Ok(0), ScheduleOutcome::Scheduled);

        let result = list_links(State(state)).await;
        let Json(links) = result.unwrap();
        
        assert_eq!(links.len(), 2);
        assert!(links.contains(&LinkDto { id: 1, url: "https://example.com/1".to_string(), date_added: time_url1.to_rfc3339() }));
        assert!(links.contains(&LinkDto { id: 2, url: "https://example.com/2".to_string(), date_added: time_url2.to_rfc3339() }));
    }

    #[tokio::test]
    async fn test_list_links_empty() {
        let state = make_state(Ok(vec![]), Ok(0), ScheduleOutcome::Scheduled);
        let result = list_links(State(state)).await;
        let Json(links) = result.unwrap();
        assert_eq!(links.len(), 0);
    }

    #[tokio::test]
    async fn test_list_links_db_error() {
        let state = make_state(Err("DB error".to_string()), Ok(0), ScheduleOutcome::Scheduled);
        let result = list_links(State(state)).await;
        let (status, Json(err)) = result.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, ApiErrorCode::DatabaseError);
    }

    #[tokio::test]
    async fn test_delete_link_success() {
        let state = make_state(Ok(vec![]), Ok(1), ScheduleOutcome::Scheduled);
        let result = delete_link(State(state), Path(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_link_not_found() {
        let state = make_state(Ok(vec![]), Ok(0), ScheduleOutcome::Scheduled);
        let result = delete_link(State(state), Path(1)).await;
        let (status, Json(err)) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, ApiErrorCode::NotFound);
    }

    #[tokio::test]
    async fn test_delete_link_db_error() {
        let state = make_state(Ok(vec![]), Err("DB error".to_string()), ScheduleOutcome::Scheduled);
        let result = delete_link(State(state), Path(1)).await;
        let (status, Json(err)) = result.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, ApiErrorCode::DatabaseError);
    }

    #[tokio::test]
    async fn test_scrape_link_scheduled() {
        let state = make_state(Ok(vec![]), Ok(0), ScheduleOutcome::Scheduled);
        let payload = ScrapeRequest { item_id: 123, days_limit: None, frequency_hours: None };
        
        let result = scrape_link(State(state), Json(payload)).await;
        let Json(resp) = result.unwrap();
        matches!(resp.state, ScrapeState::Scheduled);
    }

    #[tokio::test]
    async fn test_scrape_link_already_scheduled() {
        let state = make_state(Ok(vec![]), Ok(0), ScheduleOutcome::AlreadyInQueue);
        let payload = ScrapeRequest { item_id: 123, days_limit: None, frequency_hours: None };
        
        let result = scrape_link(State(state), Json(payload)).await;
        let Json(resp) = result.unwrap();
        matches!(resp.state, ScrapeState::AlreadyScheduled);
    }
}
