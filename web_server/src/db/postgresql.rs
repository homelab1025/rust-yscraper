use crate::CommentRecord;
use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
use async_trait::async_trait;
use chrono::Utc;
use log::debug;
use sqlx::{Pool, Postgres, QueryBuilder};

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
    async fn count_comments(&self, url_id: i64, state: Option<i32>) -> Result<u32, sqlx::Error> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM comments WHERE url_id = ");
        qb.push_bind(url_id);
        if state.is_some() {
            qb.push(" AND state = ");
            qb.push_bind(state.unwrap());
        }

        qb.build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map(|c| c as u32)
    }

    async fn page_comments(
        &self,
        offset: i64,
        count: i64,
        url_id: i64,
        state: Option<i32>,
    ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, author, date, text, url_id, state FROM comments WHERE url_id = ",
        );
        qb.push_bind(url_id);

        if let Some(s) = state {
            qb.push(" AND state = ");
            qb.push_bind(s);
        }

        qb.push(" ORDER BY date DESC, id DESC LIMIT ");
        qb.push_bind(count);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        qb.build_query_as::<DbCommentRow>()
            .fetch_all(&self.pool)
            .await
    }

    async fn upsert_comments(
        &self,
        comments: &[CommentRecord],
        url_id: i64,
    ) -> Result<usize, sqlx::Error> {
        let sql_insert = "INSERT INTO comments (id, author, date, text, url_id, state)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO UPDATE SET text=EXCLUDED.text";

        let mut inserted = 0usize;
        for comment in comments {
            let result = sqlx::query(sql_insert)
                .bind(comment.id)
                .bind(&comment.author)
                .bind(&comment.date)
                .bind(&comment.text)
                .bind(url_id)
                .bind(comment.state as i32)
                .execute(&self.pool)
                .await?;
            inserted += result.rows_affected() as usize;
        }
        Ok(inserted)
    }

    async fn update_comment_state(&self, id: i64, state: i32) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Update comment state
        sqlx::query("UPDATE comments SET state = $1 WHERE id = $2")
            .bind(state)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Update counts for the affected URL
        // TODO: this needs to be improved as it lowers performance
        sqlx::query(
            r#"
            UPDATE urls 
            SET comment_count = (SELECT COUNT(*) FROM comments WHERE url_id = (SELECT url_id FROM comments WHERE id = $1)),
                picked_comment_count = (SELECT COUNT(*) FROM comments WHERE url_id = (SELECT url_id FROM comments WHERE id = $1) AND state = 1)
            WHERE id = (SELECT url_id FROM comments WHERE id = $1)
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
impl LinksRepository for PgCommentsRepository {
    async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error> {
        sqlx::query_as::<_, DbUrlRow>(
            "SELECT id, url, date_added, comment_count, picked_comment_count FROM urls ORDER BY date_added DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn delete_link(&self, id: i64) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        debug!("Deleting comments for link id {}", id);
        let delete_comments = sqlx::query("DELETE FROM comments WHERE url_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let removed_comments = delete_comments.rows_affected();

        debug!(
            "Found {} comments to delete for link id {}",
            removed_comments, id
        );

        let deleted_links = sqlx::query("DELETE FROM urls WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        debug!(
            "Found {} link to delete for id {}",
            deleted_links.rows_affected(),
            id
        );

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
            INSERT INTO urls (id, url, date_added, frequency_hours, days_limit, comment_count, picked_comment_count)
            VALUES ($1, $2, $3, $4, $5, 0, 0)
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
        let now = Utc::now();

        sqlx::query_as::<_, ScheduledUrl>(
            r#"
            SELECT id, url, last_scraped, frequency_hours, days_limit, comment_count, picked_comment_count
            FROM urls
            WHERE (date_added + INTERVAL '1 day' * days_limit) >= $1 AND
                  (
                    last_scraped IS NULL OR 
                    last_scraped + INTERVAL '1 hour' * frequency_hours < $1
                  )
            ORDER BY last_scraped ASC NULLS FIRST
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_last_scraped(&self, url_id: i64) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE urls SET last_scraped = $1 WHERE id = $2")
            .bind(now)
            .bind(url_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_comment_count(&self, url_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE urls 
            SET comment_count = (SELECT COUNT(*) FROM comments WHERE url_id = $1),
                picked_comment_count = (SELECT COUNT(*) FROM comments WHERE url_id = $1 AND state = 1)
            WHERE id = $1
            "#,
        )
        .bind(url_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
