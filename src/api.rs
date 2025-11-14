// Access items defined in the crate root (main.rs)
use super::{AppState, CommentRecord};
use crate::scrape::get_comments as scrape_get_comments;
use crate::utils::{create_batches, extract_item_id_from_url};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::{error, info};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
pub struct CommentsQuery {
    pub offset: Option<i64>,
    pub count: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CommentDto {
    pub id: i64,
    pub text: String,
    pub user: String,
    pub url_id: i64,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct CommentsPage {
    pub total: i64,
    pub items: Vec<CommentDto>,
}

/// GET /comments — returns comments ordered by date desc with pagination
#[axum::debug_handler]
pub async fn list_comments(
    State(state): State<AppState>,
    Query(q): Query<CommentsQuery>,
) -> impl IntoResponse {
    let offset = q.offset.unwrap_or(0).max(0);
    let count = q.count.unwrap_or(10).clamp(1, 100);

    // total count
    let total = match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comments")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to count comments: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch total").into_response();
        }
    };

    // page items ordered by date desc; fallback id desc for ties
    let rows: Result<Vec<CommentDto>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, author AS user, date, text, url_id
        FROM comments
        ORDER BY date DESC, id DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(count)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await;

    let items = match rows {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch comments page: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to fetch comments",
            )
                .into_response();
        }
    };

    let body = CommentsPage { total, items };
    Json(body).into_response()
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

    // Ensure the URL is recorded in the urls table and get (or confirm) its id
    if let Err(e) = upsert_url(&state.db_pool, url_id, &target_url).await {
        error!("Failed to upsert url: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record url".to_string(),
        );
    }

    info!("/scrape called; starting scraping for {}", target_url);
    let comments_retrieval = scrape_get_comments(&target_url).await;
    match comments_retrieval {
        Ok(comments) => {
            info!("Parsed {} root comments", comments.len());

            let batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);
            let mut total_inserted = 0usize;
            for batch in batches.iter() {
                match insert_comments(&state.db_pool, batch, url_id).await {
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

async fn upsert_url(pool: &Pool<Sqlite>, id: i64, url: &str) -> Result<(), sqlx::Error> {
    // Insert if missing; if present, ignore
    sqlx::query("INSERT OR IGNORE INTO urls (id, url) VALUES (?1, ?2)")
        .bind(id)
        .bind(url)
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_comments(
    pool: &Pool<Sqlite>,
    comments: &Vec<CommentRecord>,
    url_id: i64,
) -> Result<usize, sqlx::Error> {
    let sql_insert = "INSERT INTO comments (id, author, date, text, url_id) \
    VALUES (?1, ?2, ?3, ?4, ?5) \
    ON CONFLICT (id) DO UPDATE \
    SET text=?4, url_id=?5";

    let mut inserted = 0usize;
    for comment in comments {
        let result = sqlx::query(sql_insert)
            .bind(comment.id)
            .bind(&comment.author)
            .bind(&comment.date)
            .bind(&comment.text)
            .bind(url_id)
            .execute(pool)
            .await?;
        inserted += result.rows_affected() as usize; // OR IGNORE returns 0 when skipped due to PK conflict
    }
    Ok(inserted)
}
