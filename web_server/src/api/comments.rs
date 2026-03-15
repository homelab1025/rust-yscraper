use super::common::{ApiError, ApiErrorCode};
use crate::api::app_state::AppState;
use crate::db::comments_repository::CommentsRepository;
use axum::extract::{FromRef, Json, Query, State};
use axum::http::StatusCode;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone)]
pub struct CommentsAppState {
    pub repo: Arc<dyn CommentsRepository>,
    pub config: crate::config::AppConfig,
}

impl FromRef<AppState> for CommentsAppState {
    fn from_ref(input: &AppState) -> Self {
        CommentsAppState {
            repo: input.repo.clone(),
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
    pub state: Option<crate::CommentState>,
    pub sort_by: Option<crate::SortBy>,
    pub sort_order: Option<crate::SortOrder>,
}

impl std::fmt::Display for CommentsFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CommentsFilter {{ offset: {:?}, count: {:?}, url_id: {:?}, state: {:?}, sort_by: {:?}, sort_order: {:?} }}",
            self.offset, self.count, self.url_id, self.state, self.sort_by, self.sort_order
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
    pub state: crate::CommentState,
    pub subcomment_count: i32,
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
    info!("list_comments called with {:?}", filter);
    let offset = filter.offset.unwrap_or(0).max(0);
    let count = filter.count.unwrap_or(10).clamp(1, 100);

    let url_id = filter.url_id;

    // total count
    let state_int = filter.state.map(|state| state as i32);
    let total = match state.repo.count_comments(url_id, state_int).await {
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
    let rows = match state
        .repo
        .page_comments(
            offset,
            count,
            url_id,
            state_int,
            filter.sort_by,
            filter.sort_order,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch comments page: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::DatabaseError,
                    msg: "failed to fetch comments".to_string(),
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
            state: row.state.into(),
            subcomment_count: row.subcomment_count,
        })
        .collect();

    let body = CommentsPage { total, items };
    Ok(Json(body))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStateRequest {
    pub state: crate::CommentState,
}

/// Update comment state
#[utoipa::path(
    patch,
    path = "/comments/{id}/state",
    responses(
        (status = 200, description = "Comment state updated"),
        (status = 500, description = "Database error", body = ApiError)
    ),
    params(
        ("id" = i64, Path, description = "Comment ID"),
    ),
    request_body = UpdateStateRequest
)]
pub async fn update_comment_state(
    State(state): State<CommentsAppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<UpdateStateRequest>,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    info!(
        "update_comment_state called for {} with state {:?}",
        id, payload.state
    );
    state
        .repo
        .update_comment_state(id, payload.state as i32)
        .await
        .map_err(|e| {
            error!("Failed to update comment state: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::DatabaseError,
                    msg: "failed to update comment state".to_string(),
                }),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::comments_repository::DbCommentRow;
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tokio::sync::Mutex as AsyncMutex;
    use tower::util::ServiceExt;

    #[derive(Debug, Default)]
    struct MockedRepo {
        // None -> simulate error; Some(v) -> return Ok(v)
        count_ok: AsyncMutex<Option<i64>>,
        // None -> simulate error; Some(rows) -> return Ok(clone)
        page_ok: AsyncMutex<Option<Vec<DbCommentRow>>>,
        // Records the last state passed to count/page
        last_filter_state: AsyncMutex<Option<i32>>,
        // Records the last sorting params passed to page
        last_sort_by: AsyncMutex<Option<crate::SortBy>>,
        last_sort_order: AsyncMutex<Option<crate::SortOrder>>,
        // Records the last state passed to update
        last_update_state: AsyncMutex<Option<i32>>,
    }

    impl MockedRepo {
        fn with_ok(count: i64, rows: Vec<DbCommentRow>) -> Self {
            Self {
                count_ok: AsyncMutex::new(Some(count)),
                page_ok: AsyncMutex::new(Some(rows)),
                last_filter_state: AsyncMutex::new(None),
                last_sort_by: AsyncMutex::new(None),
                last_sort_order: AsyncMutex::new(None),
                last_update_state: AsyncMutex::new(None),
            }
        }

        fn with_count_err() -> Self {
            Self {
                count_ok: AsyncMutex::new(None),
                page_ok: AsyncMutex::new(Some(vec![])),
                last_filter_state: AsyncMutex::new(None),
                last_sort_by: AsyncMutex::new(None),
                last_sort_order: AsyncMutex::new(None),
                last_update_state: AsyncMutex::new(None),
            }
        }

        fn with_page_err(total: i64) -> Self {
            Self {
                count_ok: AsyncMutex::new(Some(total)),
                page_ok: AsyncMutex::new(None),
                last_filter_state: AsyncMutex::new(None),
                last_sort_by: AsyncMutex::new(None),
                last_sort_order: AsyncMutex::new(None),
                last_update_state: AsyncMutex::new(None),
            }
        }
    }

    #[async_trait]
    impl CommentsRepository for MockedRepo {
        async fn count_comments(
            &self,
            _url_id: i64,
            state: Option<i32>,
        ) -> Result<u32, sqlx::Error> {
            *self.last_filter_state.lock().await = state;
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
            state: Option<i32>,
            sort_by: Option<crate::SortBy>,
            sort_order: Option<crate::SortOrder>,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            *self.last_filter_state.lock().await = state;
            *self.last_sort_by.lock().await = sort_by;
            *self.last_sort_order.lock().await = sort_order;
            match &*self.page_ok.lock().await {
                Some(rows) => Ok(rows.clone()),
                None => Err(sqlx::Error::RowNotFound),
            }
        }

        async fn upsert_comments(
            &self,
            _comments: &[crate::CommentRecord],
            _url_id: i64,
            _thread_month: Option<i32>,
            _thread_year: Option<i32>,
        ) -> Result<usize, sqlx::Error> {
            // Default to success for tests that don't exercise DB
            Ok(0)
        }

        async fn update_comment_state(&self, _id: i64, state: i32) -> Result<(), sqlx::Error> {
            *self.last_update_state.lock().await = Some(state);
            Ok(())
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
    }

    fn make_test_config() -> crate::config::AppConfig {
        crate::config::AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
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
            state: 0,
            subcomment_count: 0,
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn clamps_count_below_min_to_1_indirectly() {
        let rows: Vec<DbCommentRow> = vec![make_comment_row(1, "a", "2024-01-01", "t", 9)];
        let repo = Arc::new(MockedRepo::with_ok(5, rows));
        let state = State(CommentsAppState {
            repo,
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: Some(0),
            count: Some(0),
            url_id: 9,
            state: None,
            sort_by: None,
            sort_order: None,
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
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: Some(0),
            count: Some(1000),
            url_id: 2,
            state: None,
            sort_by: None,
            sort_order: None,
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
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
            state: None,
            sort_by: None,
            sort_order: None,
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
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
            state: None,
            sort_by: None,
            sort_order: None,
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
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 77,
            state: None,
            sort_by: None,
            sort_order: None,
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
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
            state: None,
            sort_by: None,
            sort_order: None,
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
            config: make_test_config(),
        });
        let query = Query(CommentsFilter {
            offset: Some(0),
            count: Some(10),
            url_id: 1,
            state: None,
            sort_by: None,
            sort_order: None,
        });
        let resp = list_comments(state, query).await;
        let err = resp.unwrap_err();
        assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.1.code, ApiErrorCode::DatabaseError);
        // Handler currently returns the same message for page error as count error
        assert_eq!(err.1.msg, "failed to fetch comments".to_string());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn list_comments_handles_state_filter() {
        let repo = Arc::new(MockedRepo::with_ok(0, vec![]));
        let state = State(CommentsAppState {
            repo: repo.clone(),
            config: make_test_config(),
        });

        // Test with state = Picked
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
            state: Some(crate::CommentState::Picked),
            sort_by: None,
            sort_order: None,
        });

        let resp = list_comments(state.clone(), query).await;
        assert!(resp.is_ok());
        assert_eq!(*repo.last_filter_state.lock().await, Some(1));

        // Test with state = Discarded
        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
            state: Some(crate::CommentState::Discarded),
            sort_by: None,
            sort_order: None,
        });

        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        assert_eq!(*repo.last_filter_state.lock().await, Some(2));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn list_comments_handles_sorting_params() {
        let repo = Arc::new(MockedRepo::with_ok(0, vec![]));
        let state = State(CommentsAppState {
            repo: repo.clone(),
            config: make_test_config(),
        });

        let query = Query(CommentsFilter {
            offset: None,
            count: None,
            url_id: 1,
            state: None,
            sort_by: Some(crate::SortBy::SubcommentCount),
            sort_order: Some(crate::SortOrder::Asc),
        });

        let resp = list_comments(state, query).await;
        assert!(resp.is_ok());
        assert_eq!(
            *repo.last_sort_by.lock().await,
            Some(crate::SortBy::SubcommentCount)
        );
        assert_eq!(
            *repo.last_sort_order.lock().await,
            Some(crate::SortOrder::Asc)
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn update_comment_state_converts_to_int() {
        let repo = Arc::new(MockedRepo::default());
        let state = State(CommentsAppState {
            repo: repo.clone(),
            config: make_test_config(),
        });

        // Test update to Picked
        let req = Json(UpdateStateRequest {
            state: crate::CommentState::Picked,
        });
        let resp = update_comment_state(state.clone(), axum::extract::Path(42), req).await;
        assert!(resp.is_ok());
        assert_eq!(*repo.last_update_state.lock().await, Some(1));

        // Test update to Discarded
        let req = Json(UpdateStateRequest {
            state: crate::CommentState::Discarded,
        });
        let resp = update_comment_state(state, axum::extract::Path(42), req).await;
        assert!(resp.is_ok());
        assert_eq!(*repo.last_update_state.lock().await, Some(2));
    }

    // TODO: should this actually be the way to do integration tests for the api?
    #[tokio::test(flavor = "current_thread")]
    async fn update_comment_state_returns_422_for_unknown_string() {
        let repo = Arc::new(MockedRepo::default());
        let app_state = CommentsAppState {
            repo: repo.clone(),
            config: make_test_config(),
        };

        let app = axum::Router::new()
            .route(
                "/comments/{id}/state",
                axum::routing::patch(update_comment_state),
            )
            .with_state(app_state);

        let json = r#"{"state": "UNKNOWN"}"#;
        let req = Request::builder()
            .method("PATCH")
            .uri("/comments/42/state")
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let resp = ServiceExt::<Request<Body>>::oneshot(app, req)
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
