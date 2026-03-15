use async_trait::async_trait;

#[async_trait]
pub trait LinksRepository: Send + Sync {
    /// Returns all links (urls) with their IDs and added date.
    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error>;

    /// Returns the number of deleted rows.
    /// Returns 0 if the link with the given ID does not exist.
    async fn delete_link(&self, id: i64) -> Result<u64, sqlx::Error>;

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

}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbUrlRow {
    pub id: i64,
    pub url: String,
    pub date_added: chrono::DateTime<chrono::Utc>,
    pub comment_count: i32,
    pub picked_comment_count: i32,
    pub thread_month: Option<i32>,
    pub thread_year: Option<i32>,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct ScheduledUrl {
    pub id: i64,
    pub url: String,
    // when it was last successfully scraped
    pub last_scraped: Option<chrono::DateTime<chrono::Utc>>,
    // hours between scrapes
    pub frequency_hours: i32,
    // for how many days to refresh comments
    pub days_limit: i32,
    pub comment_count: i32,
    pub picked_comment_count: i32,
    pub thread_month: Option<i32>,
    pub thread_year: Option<i32>,
}
