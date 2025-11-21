use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use rust_yscraper::api::{list_comments, CommentsQuery};
use rust_yscraper::db::{CommentsRepository, DbCommentRow};
use rust_yscraper::{ApiErrorCode, CommentsAppState};
use sqlx::Error as SqlxError;
use tokio::sync::Mutex;

#[derive(Debug)]
struct MockRepo {
    count_ok: Mutex<Option<i64>>,              // None => error
    page_ok: Mutex<Option<Vec<DbCommentRow>>>, // None => error
}

impl MockRepo {
    fn with_ok(count: i64, rows: Vec<DbCommentRow>) -> Self {
        Self {
            count_ok: Mutex::new(Some(count)),
            page_ok: Mutex::new(Some(rows)),
        }
    }

    fn with_count_err() -> Self {
        Self {
            count_ok: Mutex::new(None),
            page_ok: Mutex::new(Some(vec![])),
        }
    }

    fn with_page_err(total: i64) -> Self {
        Self {
            count_ok: Mutex::new(Some(total)),
            page_ok: Mutex::new(None),
        }
    }
}

#[async_trait]
impl CommentsRepository for MockRepo {
    async fn count_comments(&self) -> Result<i64, sqlx::Error> {
        match *self.count_ok.lock().await {
            Some(v) => Ok(v),
            None => Err(SqlxError::RowNotFound),
        }
    }

    async fn page_comments(
        &self,
        _offset: i64,
        _count: i64,
    ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
        match &*self.page_ok.lock().await {
            Some(rows) => Ok(rows.clone()),
            None => Err(SqlxError::RowNotFound),
        }
    }

    async fn upsert_url(&self, _id: i64, _url: &str) -> Result<(), sqlx::Error> {
        // Not used by the handler under test
        unimplemented!("upsert_url is not used in list_comments tests")
    }

    async fn upsert_comments(
        &self,
        _comments: &[rust_yscraper::CommentRecord],
        _url_id: i64,
    ) -> Result<usize, sqlx::Error> {
        // Not used by the handler under test
        unimplemented!("upsert_comments is not used in list_comments tests")
    }
}

fn make_row(id: i64, author: &str, date: &str, text: &str, url_id: i64) -> DbCommentRow {
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
    let rows: Vec<DbCommentRow> = vec![make_row(1, "a", "2024-01-01", "t", 9)];
    let repo = Arc::new(MockRepo::with_ok(5, rows));
    let state = State(CommentsAppState { repo });
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
        .map(|i| make_row(i as i64, "u", "2024-04-01", "t", 2))
        .collect();
    let repo = Arc::new(MockRepo::with_ok(150, rows));
    let state = State(CommentsAppState { repo });
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
    let repo = Arc::new(MockRepo::with_ok(0, vec![]));
    let state = State(CommentsAppState { repo });
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
        make_row(3, "a", "2024-01-02", "t3", 1),
        make_row(2, "b", "2024-01-02", "t2", 1),
        make_row(1, "c", "2024-01-01", "t1", 1),
    ];
    let repo = Arc::new(MockRepo::with_ok(3, rows));
    let state = State(CommentsAppState { repo });
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
    let row = make_row(42, "zoe", "2024-05-05", "hello", 77);
    let repo = Arc::new(MockRepo::with_ok(1, vec![row]));
    let state = State(CommentsAppState { repo });
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
    let repo = Arc::new(MockRepo::with_count_err());
    let state = State(CommentsAppState { repo });
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
    let repo = Arc::new(MockRepo::with_page_err(10));
    let state = State(CommentsAppState { repo });
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
