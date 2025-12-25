use async_trait::async_trait;

#[async_trait]
pub trait LinksRepository: Send + Sync {
    /// Returns all links (urls) with their IDs and added date.
    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error>;
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbUrlRow {
    pub id: i64,
    pub url: String,
    pub date_added: chrono::DateTime<chrono::Utc>,
}