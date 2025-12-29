use async_trait::async_trait;

#[async_trait]
pub trait LinksRepository: Send + Sync {
    /// Returns all links (urls) with their IDs and added date.
    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error>;
    
    /// Returns the number of deleted rows.
    /// Returns 0 if the link with the given ID does not exist.
    async fn delete_link(&self, id: i64) -> Result<u64, sqlx::Error>;
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DbUrlRow {
    pub id: i64,
    pub url: String,
    pub date_added: chrono::DateTime<chrono::Utc>,
}
