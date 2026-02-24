use crate::CommentRecord;
use async_trait::async_trait;

#[async_trait]
pub trait CommentsRepository: Send + Sync {
    /// Total number of comments for a specific url_id, optionally filtered by state.
    async fn count_comments(&self, url_id: i64, state: Option<i32>) -> Result<u32, sqlx::Error>;

    /// Returns a page of comments ordered by date desc, id desc, filtered by url_id and optionally state.
    async fn page_comments(
        &self,
        offset: i64,
        count: i64,
        url_id: i64,
        state: Option<i32>,
    ) -> Result<Vec<DbCommentRow>, sqlx::Error>;

    /// Insert or update a batch of comments for a given url_id.
    async fn upsert_comments(
        &self,
        comments: &[CommentRecord],
        url_id: i64,
    ) -> Result<usize, sqlx::Error>;

    /// Update the state of a specific comment.
    async fn update_comment_state(&self, id: i64, state: i32) -> Result<(), sqlx::Error>;
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbCommentRow {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub url_id: i64,
    pub state: i32,
}
