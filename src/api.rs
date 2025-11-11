// Access items defined in the crate root (main.rs)
use super::{AppState, CommentRecord};
use crate::scrape::get_comments;
use crate::utils::create_batches;
use axum::{extract::State, response::IntoResponse};
use log::{error, info};
use sqlx::{Pool, Sqlite};

/// Health check handler
pub async fn ping() -> impl IntoResponse {
    "pong"
}

/// Triggers scraping and inserts results into the database.
pub async fn scrape_handler(State(state): State<AppState>) -> impl IntoResponse {
    info!("/scrape called; starting scraping for {}", state.url);
    let comments = get_comments(&state.url).await;
    info!("Parsed {} root comments", comments.len());

    let batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);
    let mut total_inserted = 0usize;
    for batch in batches.iter() {
        match insert_comments(&state.db_pool, batch).await {
            Ok(n) => {
                total_inserted += n;
                info!("Inserted {} comments into the database", n);
            }
            Err(e) => error!("Failed to insert comments: {}", e),
        }
    }

    format!("ok: parsed={}, inserted={}", comments.len(), total_inserted)
}

async fn insert_comments(
    pool: &Pool<Sqlite>,
    comments: &Vec<CommentRecord>,
) -> Result<usize, sqlx::Error> {
    let mut inserted = 0usize;
    for c in comments {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO comments (id, author, date, text) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(c.id)
        .bind(&c.author)
        .bind(&c.date)
        .bind(&c.text)
        .execute(pool)
        .await?;
        inserted += result.rows_affected() as usize; // OR IGNORE returns 0 when skipped due to PK conflict
    }
    Ok(inserted)
}
