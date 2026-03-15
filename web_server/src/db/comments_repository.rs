use crate::CommentRecord;
use async_trait::async_trait;

#[async_trait]
pub trait CommentsRepository: Send + Sync {
    /// Total number of comments for a specific url_id, optionally filtered by state.
    async fn count_comments(&self, url_id: i64, state: Option<i32>) -> Result<u32, sqlx::Error>;

    /// Returns a page of comments filtered by url_id and optionally state, with configurable sorting.
    async fn page_comments(
        &self,
        offset: i64,
        count: i64,
        url_id: i64,
        state: Option<i32>,
        sort_by: Option<crate::SortBy>,
        sort_order: Option<crate::SortOrder>,
    ) -> Result<Vec<DbCommentRow>, sqlx::Error>;

    /// Insert or update a batch of comments for a given url_id, and atomically
    /// update the URL's counts and thread metadata.
    async fn upsert_comments(
        &self,
        comments: &[CommentRecord],
        url_id: i64,
        thread_month: Option<i32>,
        thread_year: Option<i32>,
    ) -> Result<usize, sqlx::Error>;

    /// Update the state of a specific comment.
    async fn update_comment_state(&self, id: i64, state: i32) -> Result<(), sqlx::Error>;

    /// Fetch a single comment by its ID.
    async fn get_comment(&self, id: i64) -> Result<Option<DbCommentRow>, sqlx::Error>;
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbCommentRow {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub url_id: i64,
    pub state: i32,
    pub subcomment_count: i32,
}
