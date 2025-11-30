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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::app_state::CommentsAppState;
    use crate::api::scrape_task::ScrapeTask;
    use crate::db::{CommentsRepository, DbCommentRow};
    use async_trait::async_trait;
    use reqwest::Client;
    use std::fmt::Debug;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc::error::TrySendError;

    #[derive(Clone, Default)]
    struct StubRepo;

    #[async_trait]
    impl CommentsRepository for StubRepo {
        async fn count_comments(&self) -> Result<i64, sqlx::Error> {
            Ok(0)
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            Ok(vec![])
        }

        async fn upsert_url(&self, _id: i64, _url: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn upsert_comments(
            &self,
            _comments: &[crate::CommentRecord],
            _url_id: i64,
        ) -> Result<usize, sqlx::Error> {
            Ok(0)
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum ScheduleOutcome {
        Scheduled,
        AlreadyInQueue,
        Error,
    }

    struct StubScheduler {
        outcome: ScheduleOutcome,
        last_task: Mutex<Option<ScrapeTask>>,
    }

    impl StubScheduler {
        fn new(outcome: ScheduleOutcome) -> Self {
            Self {
                outcome,
                last_task: Mutex::new(None),
            }
        }

        fn last_task(&self) -> Option<ScrapeTask> {
            self.last_task.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl crate::task_queue::TaskScheduler<ScrapeTask> for StubScheduler {
        async fn schedule(&self, task: ScrapeTask) -> Result<bool, TrySendError<ScrapeTask>> {
            self.last_task.lock().unwrap().replace(task.clone());
            match self.outcome {
                ScheduleOutcome::Scheduled => Ok(true),
                ScheduleOutcome::AlreadyInQueue => Ok(false),
                ScheduleOutcome::Error => Err(TrySendError::Full(task)),
            }
        }
    }

    fn make_state(
        outcome: ScheduleOutcome,
    ) -> (CommentsAppState, Arc<StubScheduler>, Arc<StubRepo>) {
        let repo = Arc::new(StubRepo);
        let client = Arc::new(Client::new());
        let scheduler = Arc::new(StubScheduler::new(outcome));

        let state = CommentsAppState {
            repo: repo.clone(),
            http_client: client,
            task_queue: scheduler.clone() as Arc<dyn crate::task_queue::TaskScheduler<ScrapeTask>>,
        };

        (state, scheduler, repo)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn schedules_task_and_returns_scheduled() {
        let (state, stub_sched, repo) = make_state(ScheduleOutcome::Scheduled);
        let item_id = 12345_i64;
        let payload = ScrapeRequest { item_id };

        let res = scrape_comments(State(state), Json(payload)).await;

        // Assert response
        let Json(body) = res.expect("expected Ok(Json)");
        match body.state {
            ScrapeState::Scheduled => {}
            _ => panic!("expected Scheduled state"),
        }

        // Assert the scheduled task contents
        let captured = stub_sched
            .last_task()
            .expect("task should have been scheduled");
        let expected_url = format!("https://news.ycombinator.com/item?id={}", item_id);
        let expected = ScrapeTask::new(expected_url, item_id, repo);
        assert!(captured == expected, "ScrapeTask fields should match");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn already_scheduled_maps_to_already_scheduled_state() {
        let (state, _stub_sched, _repo) = make_state(ScheduleOutcome::AlreadyInQueue);
        let payload = ScrapeRequest { item_id: 77 };

        let res = scrape_comments(State(state), Json(payload)).await;
        let Json(body) = res.expect("expected Ok(Json)");
        match body.state {
            ScrapeState::AlreadyScheduled => {}
            _ => panic!("expected AlreadyScheduled state"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn scheduling_error_maps_to_500_and_api_error() {
        let (state, _stub_sched, _repo) = make_state(ScheduleOutcome::Error);
        let payload = ScrapeRequest { item_id: 5 };

        let res = scrape_comments(State(state), Json(payload)).await;
        let (status, Json(err)) = res.expect_err("expected error response");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, ApiErrorCode::SchedulingError);
        assert_eq!(err.msg, "could not schedule scrape task");
    }
}
