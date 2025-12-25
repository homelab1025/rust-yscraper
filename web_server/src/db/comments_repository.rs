use crate::CommentRecord;
use async_trait::async_trait;

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
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbCommentRow {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub url_id: i64,
}
