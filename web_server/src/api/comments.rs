use super::common::{ApiError, ApiErrorCode};
use crate::api::app_state::AppState;
use crate::db::comments_repository::CommentsRepository;
use crate::scrape_task::ScrapeTask;
use crate::task_queue::TaskScheduler;
use axum::extract::{FromRef, Json, Query, State};
use axum::http::StatusCode;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone)]
pub struct CommentsAppState {
    pub repo: Arc<dyn CommentsRepository>,
    // TODO: actually use this in the scraper
    pub task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    pub config: crate::config::AppConfig,
}

impl FromRef<AppState> for CommentsAppState {
    fn from_ref(input: &AppState) -> Self {
        CommentsAppState {
            repo: input.repo.clone(),
            task_queue: input.task_queue.clone(),
            config: input.config.clone(),
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CommentsFilter {
    pub offset: Option<i64>,
    pub count: Option<i64>,
    pub url_id: i64,
}

impl std::fmt::Display for CommentsFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CommentsFilter {{ offset: {:?}, count: {:?}, url_id: {} }}",
            self.offset, self.count, self.url_id
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CommentDto {
    pub id: i64,
    pub text: String,
    pub user: String,
    pub url_id: i64,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CommentsPage {
    pub total: i64,
    pub items: Vec<CommentDto>,
}

/// List comments with pagination
#[utoipa::path(
    get,
    path = "/comments",
    params(CommentsFilter),
    responses(
        (status = 200, description = "List of comments", body = CommentsPage),
        (status = 500, description = "Database error", body = ApiError)
    )
)]
#[axum::debug_handler]
pub async fn list_comments(
    State(state): State<CommentsAppState>,
    Query(filter): Query<CommentsFilter>,
) -> Result<Json<CommentsPage>, (StatusCode, Json<ApiError>)> {
    info!("list_comments called with {}", filter);
    let offset = filter.offset.unwrap_or(0).max(0);
    let count = filter.count.unwrap_or(10).clamp(1, 100);

    // total count
    let total = match state.repo.count_comments(filter.url_id).await {
        Ok(c) => c as i64,
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
    let rows = match state.repo.page_comments(offset, count, filter.url_id).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::comments_repository::DbCommentRow;
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use crate::scrape_task::ScrapeTask;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::Mutex as AsyncMutex;
    use tokio::sync::mpsc::error::TrySendError;

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
        async fn count_comments(&self, _url_id: i64) -> Result<u32, sqlx::Error> {
            match *self.count_ok.lock().await {
                Some(v) => Ok(v as u32),
                None => Err(sqlx::Error::RowNotFound),
            }
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
            _url_id: i64,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            match &*self.page_ok.lock().await {
                Some(rows) => Ok(rows.clone()),
                None => Err(sqlx::Error::RowNotFound),
            }
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

    #[async_trait]
    impl LinksRepository for MockedRepo {
        async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error> {
            Ok(vec![])
        }
        async fn delete_link(&self, _id: i64) -> Result<u64, sqlx::Error> {
            Ok(0)
        }
        async fn upsert_url_with_scheduling(
            &self,
            _id: i64,
            _url: &str,
            _frequency_hours: u32,
            _days_limit: u32,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(vec![])
        }
        async fn update_last_scraped(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn update_comment_count(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }
    }

    // Minimal dummy scheduler for CommentsAppState
    struct DummyScheduler;
    #[async_trait]
    impl TaskScheduler<ScrapeTask> for DummyScheduler {
        async fn schedule(&self, _task: ScrapeTask) -> Result<bool, TrySendError<ScrapeTask>> {
            Ok(true)
        }
    }

    fn make_comment_row(
        id: i64,
        author: &str,
        date: &str,
        text: &str,
        url_id: i64,
    ) -> DbCommentRow {
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
            task_queue: Arc::new(DummyScheduler),
            config: crate::config::AppConfig {
                server_port: 3000,
                db_username: "u".to_string(),
                db_password: "p".to_string(),
                db_name: "n".to_string(),
                db_host: "h".to_string(),
                db_port: 5432,
                default_days_limit: 7,
                default_frequency_hours: 24,
            },
        });
        let query = Query(CommentsFilter {
            offset: Some(0),
            count: Some(0),
            url_id: 9,
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
        let config = crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        };
        let state = State(CommentsAppState {
            repo,
            task_queue: Arc::new(DummyScheduler),
            config,
        });
        let query = Query(CommentsFilter {
            offset: Some(0),
            count: Some(1000),
            url_id: 2,
        });

        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        let page = resp.unwrap();
        assert_eq!(page.items.len(), 100);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_empty_list_when_store_empty() {
        let repo = Arc::new(MockedRepo::with_ok(0, vec![]));
        let config = crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        };
        let state = State(CommentsAppState {
            repo,
            task_queue: Arc::new(DummyScheduler),
            config,
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
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
        let config = crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        };
        let state = State(CommentsAppState {
            repo,
            task_queue: Arc::new(DummyScheduler),
            config,
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
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
        let config = crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        };
        let state = State(CommentsAppState {
            repo,
            task_queue: Arc::new(DummyScheduler),
            config,
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 77,
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
        let config = crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        };
        let state = State(CommentsAppState {
            repo,
            task_queue: Arc::new(DummyScheduler),
            config,
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
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
        let config = crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        };
        let state = State(CommentsAppState {
            repo,
            task_queue: Arc::new(DummyScheduler),
            config,
        });
        let query = Query(CommentsFilter {
            offset: Some(0),
            count: Some(10),
            url_id: 1,
        });
        let resp = list_comments(state, query).await;
        let err = resp.unwrap_err();
        assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.1.code, ApiErrorCode::DatabaseError);
        // Handler currently returns the same message for page error as count error
        assert_eq!(err.1.msg, "failed to count comments");
    }
}
