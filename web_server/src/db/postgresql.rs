use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
use crate::db::links_repository::{DbUrlRow, LinksRepository};
use crate::CommentRecord;
use async_trait::async_trait;
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
}
