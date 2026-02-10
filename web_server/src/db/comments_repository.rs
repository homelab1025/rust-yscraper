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

    /// Insert or update a batch of comments for a given url_id.
    async fn upsert_comments(
        &self,
        comments: &[CommentRecord],
        url_id: i64,
    ) -> Result<usize, sqlx::Error>;

    /// Update URL scheduling metadata
    async fn upsert_url_with_scheduling(
        &self,
        id: i64,
        url: &str,
        frequency_hours: u32,
        days_limit: u32,
    ) -> Result<(), sqlx::Error>;

    /// Get URLs that are due for refresh based on scheduling
    async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error>;

    /// Update last_scraped timestamp for a URL
    async fn update_last_scraped(&self, url_id: i64) -> Result<(), sqlx::Error>;
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbCommentRow {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub url_id: i64,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct ScheduledUrl {
    pub id: i64,
    pub url: String,
    pub last_scraped: Option<chrono::DateTime<chrono::Utc>>,
    pub frequency_hours: i32,
    pub days_limit: i32,
}
