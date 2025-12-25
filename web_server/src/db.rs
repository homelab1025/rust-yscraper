use crate::CommentRecord;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};

/// Database abstraction for comments and related URL records.
///
/// Handlers depend on this trait to decouple from a specific database.
#[async_trait]
pub trait CommentsRepository: Send + Sync {
    /// Total number of comments in the store.
    async fn count_comments(&self) -> Result<i64, sqlx::Error>;

    /// Returns a page of comments ordered by date desc, id desc.
    async fn page_comments(
        &self,
        offset: i64,
        count: i64,
    ) -> Result<Vec<DbCommentRow>, sqlx::Error>;

    /// Insert the URL if missing; no-op if present.
    async fn upsert_url(&self, id: i64, url: &str) -> Result<(), sqlx::Error>;

    /// Insert or update a batch of comments for a given url_id.
    async fn upsert_comments(
        &self,
        comments: &[CommentRecord],
        url_id: i64,
    ) -> Result<usize, sqlx::Error>;

    /// Returns all links (urls) with their IDs and added date.
    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error>;
}

/// Row type returned by repository for link listings.
#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbUrlRow {
    pub id: i64,
    pub url: String,
    pub date_added: chrono::DateTime<chrono::Utc>,
}

/// Row type returned by repository for comment listings.
#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbCommentRow {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub url_id: i64,
}

/// PostgreSQL implementation of `CommentsRepository`.
pub struct PgCommentsRepository {
    pool: Pool<Postgres>,
}

impl PgCommentsRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentsRepository for PgCommentsRepository {
    async fn count_comments(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comments")
            .fetch_one(&self.pool)
            .await
    }

    async fn page_comments(
        &self,
        offset: i64,
        count: i64,
    ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
        sqlx::query_as::<_, DbCommentRow>(
            r#"
            SELECT id, author, date, text, url_id
            FROM comments
            ORDER BY date DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(count)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert_url(&self, id: i64, url: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(id)
            .bind(url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn upsert_comments(
        &self,
        comments: &[CommentRecord],
        url_id: i64,
    ) -> Result<usize, sqlx::Error> {
        let sql_insert = "INSERT INTO comments (id, author, date, text, url_id)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET text=EXCLUDED.text, url_id=EXCLUDED.url_id";

        let mut inserted = 0usize;
        for comment in comments {
            let result = sqlx::query(sql_insert)
                .bind(comment.id)
                .bind(&comment.author)
                .bind(&comment.date)
                .bind(&comment.text)
                .bind(url_id)
                .execute(&self.pool)
                .await?;
            inserted += result.rows_affected() as usize;
        }
        Ok(inserted)
    }

    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error> {
        sqlx::query_as::<_, DbUrlRow>("SELECT id, url, date_added FROM urls ORDER BY date_added DESC")
            .fetch_all(&self.pool)
            .await
    }
}
