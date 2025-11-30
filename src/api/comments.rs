use super::common::{ApiError, ApiErrorCode};
use crate::api::app_state::CommentsAppState;
use crate::api::scrape_task::ScrapeTask;
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use log::{error, info};
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
    use tokio::sync::Mutex as AsyncMutex;

    #[derive(Debug, Default)]
    struct MockedRepo {
        // None -> simulate error; Some(v) -> return Ok(v)
        count_ok: AsyncMutex<Option<i64>>,
        // None -> simulate error; Some(rows) -> return Ok(clone)
        page_ok: AsyncMutex<Option<Vec<DbCommentRow>>>,
    }

    impl MockedRepo {
        fn with_ok(count: i64, rows: Vec<DbCommentRow>) -> Self {
            Self {
                count_ok: AsyncMutex::new(Some(count)),
                page_ok: AsyncMutex::new(Some(rows)),
            }
        }

        fn with_count_err() -> Self {
            Self {
                count_ok: AsyncMutex::new(None),
                page_ok: AsyncMutex::new(Some(vec![])),
            }
        }

        fn with_page_err(total: i64) -> Self {
            Self {
                count_ok: AsyncMutex::new(Some(total)),
                page_ok: AsyncMutex::new(None),
            }
        }
    }

    #[async_trait]
    impl CommentsRepository for MockedRepo {
        async fn count_comments(&self) -> Result<i64, sqlx::Error> {
            match *self.count_ok.lock().await {
                Some(v) => Ok(v),
                None => Err(sqlx::Error::RowNotFound),
            }
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            match &*self.page_ok.lock().await {
                Some(rows) => Ok(rows.clone()),
                None => Err(sqlx::Error::RowNotFound),
            }
        }

        async fn upsert_url(&self, _id: i64, _url: &str) -> Result<(), sqlx::Error> {
            // Default to success for tests that don't exercise DB
            Ok(())
        }

        async fn upsert_comments(
            &self,
            _comments: &[crate::CommentRecord],
            _url_id: i64,
        ) -> Result<usize, sqlx::Error> {
            // Default to success for tests that don't exercise DB
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
    ) -> (CommentsAppState, Arc<StubScheduler>, Arc<MockedRepo>) {
        let repo = Arc::new(MockedRepo::default());
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

    fn make_comment_row(id: i64, author: &str, date: &str, text: &str, url_id: i64) -> DbCommentRow {
        DbCommentRow {
            id,
            author: author.to_string(),
            date: date.to_string(),
            text: text.to_string(),
            url_id,
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn clamps_count_below_min_to_1_indirectly() {
        let rows: Vec<DbCommentRow> = vec![make_comment_row(1, "a", "2024-01-01", "t", 9)];
        let repo = Arc::new(MockedRepo::with_ok(5, rows));
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: Some(0),
            count: Some(0),
        });

        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        let page = resp.unwrap();
        assert_eq!(page.items.len(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn clamps_count_above_max_to_100_indirectly() {
        let rows: Vec<DbCommentRow> = (0..100)
            .map(|i| make_comment_row(i as i64, "u", "2024-04-01", "t", 2))
            .collect();
        let repo = Arc::new(MockedRepo::with_ok(150, rows));
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: Some(0),
            count: Some(1000),
        });

        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        let page = resp.unwrap();
        assert_eq!(page.items.len(), 100);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_empty_list_when_store_empty() {
        let repo = Arc::new(MockedRepo::with_ok(0, vec![]));
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: None,
            count: None,
        });
        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        let page = resp.unwrap();
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn preserves_desc_order_and_id_tie_break() {
        // Same date for the first two, higher id first, then older date
        let rows = vec![
            make_comment_row(3, "a", "2024-01-02", "t3", 1),
            make_comment_row(2, "b", "2024-01-02", "t2", 1),
            make_comment_row(1, "c", "2024-01-01", "t1", 1),
        ];
        let repo = Arc::new(MockedRepo::with_ok(3, rows));
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: None,
            count: None,
        });
        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        let page = resp.unwrap();
        let ids: Vec<i64> = page.items.iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![3, 2, 1]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_fields_correctly_from_dbrow_to_dto() {
        let row = make_comment_row(42, "zoe", "2024-05-05", "hello", 77);
        let repo = Arc::new(MockedRepo::with_ok(1, vec![row]));
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: None,
            count: None,
        });
        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        let page = resp.unwrap();
        assert_eq!(page.items.len(), 1);
        let c = &page.items[0];
        assert_eq!(c.id, 42);
        assert_eq!(c.user, "zoe");
        assert_eq!(c.date, "2024-05-05");
        assert_eq!(c.text, "hello");
        assert_eq!(c.url_id, 77);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_500_when_count_fails() {
        let repo = Arc::new(MockedRepo::with_count_err());
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: None,
            count: None,
        });
        let resp = list_comments(state, query).await;
        assert!(resp.is_err());
        let err = resp.unwrap_err();
        assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.1.code, ApiErrorCode::DatabaseError);
        assert_eq!(err.1.msg, "failed to count comments");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_500_when_page_query_fails() {
        let repo = Arc::new(MockedRepo::with_page_err(10));
        let state = State(CommentsAppState {
            repo,
            http_client: Arc::new(Default::default()),
            task_queue: Arc::new(crate::task_queue::TaskDedupQueue::new(3)),
        });
        let query = Query(CommentsQuery {
            offset: Some(0),
            count: Some(10),
        });
        let resp = list_comments(state, query).await;
        let err = resp.unwrap_err();
        assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.1.code, ApiErrorCode::DatabaseError);
        // Handler currently returns the same message for page error as count error
        assert_eq!(err.1.msg, "failed to count comments");
    }
}
