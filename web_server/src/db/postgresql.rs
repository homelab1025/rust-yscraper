use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
use crate::CommentRecord;
use async_trait::async_trait;
use chrono::Utc;
use log::{debug, warn};
use sqlx::{Pool, Postgres};

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
}

#[async_trait]
impl LinksRepository for PgCommentsRepository {
    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error> {
        sqlx::query_as::<_, DbUrlRow>(
            "SELECT id, url, date_added FROM urls ORDER BY date_added DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn delete_link(&self, id: i64) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let deleted_links = sqlx::query("DELETE FROM urls WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        debug!(
            "Found {} link to delete for id {}",
            deleted_links.rows_affected(),
            id
        );

        let delete_comments = sqlx::query("DELETE FROM comments WHERE url_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let removed_comments = delete_comments.rows_affected();

        debug!(
            "Found {} comments to delete for link id {}",
            removed_comments, id
        );
        if removed_comments > 0 && deleted_links.rows_affected() == 0 {
            warn!(
                "There were {} comments for link id {} but no link was found. This should not happen.",
                removed_comments, id
            );
        }

        tx.commit().await?;

        Ok(deleted_links.rows_affected())
    }

    async fn upsert_url_with_scheduling(
        &self,
        id: i64,
        url: &str,
        frequency_hours: u32,
        days_limit: u32,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        // Insert or update URL with scheduling metadata
        // Note: Only update URL field on conflict to preserve original scheduling values
        sqlx::query(
            r#"
            INSERT INTO urls (id, url, last_scraped, frequency_hours, days_limit)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                url = EXCLUDED.url
            "#,
        )
        .bind(id)
        .bind(url)
        .bind(now)
        .bind(frequency_hours as i32)
        .bind(days_limit as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
        let _now = Utc::now();

        sqlx::query_as::<_, ScheduledUrl>(
            r#"
            SELECT id, url, last_scraped, frequency_hours, days_limit
            FROM urls
            WHERE date_added >= (NOW() - INTERVAL '1 day' * days_limit) AND
                  ((last_scraped IS NOT NULL AND last_scraped < NOW() - INTERVAL '1 hour' * frequency_hours) OR (last_scraped IS NULL))
            ORDER BY last_scraped ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn update_last_scraped(&self, url_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE urls SET last_scraped = NOW() WHERE id = $1")
            .bind(url_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
