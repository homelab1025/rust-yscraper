mod scrape;

use crate::scrape::get_comments;
use config::{Config, File, FileFormat};
use log::{error, info};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

const DEFAULT_URL: &str = "https://news.ycombinator.com/item?id=45561428";
const CONFIG_PATH: &str = "config.properties";

#[derive(Debug, Default, Clone)]
pub struct CommentRecord {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub tags: Vec<String>,
}

async fn init_db(db_path: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let db_url = format!("sqlite://{}", db_path);
    if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
        info!("Initializing database at {}", db_path);
        Sqlite::create_database(&db_path).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author TEXT NOT NULL,
            date TEXT NOT NULL,
            text TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn insert_comments(
    pool: &Pool<Sqlite>,
    comments: &Vec<CommentRecord>,
) -> Result<usize, sqlx::Error> {
    let mut inserted = 0usize;
    for c in comments {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO comments (id, author, date, text) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(c.id)
        .bind(&c.author)
        .bind(&c.date)
        .bind(&c.text)
        .execute(pool)
        .await?;
        inserted += result.rows_affected() as usize; // OR IGNORE returns 0 when skipped due to PK conflict
    }
    Ok(inserted)
}

async fn get_comment_count(pool: &Pool<Sqlite>) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Split a slice of items into consecutive batches of size `batches_count`.
/// If `batches_count` is 0, returns an empty vector.
fn create_batches<T: Clone>(items: &[T], batches_count: usize) -> Vec<Vec<T>> {
    if batches_count == 0 {
        return Vec::new();
    }
    items
        .chunks(batches_count)
        .map(|chunk| chunk.to_vec())
        .collect()
}

#[tokio::main]
async fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    let db_path = "comments.db";
    let pool = match init_db(db_path).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return;
        }
    };

    match get_comment_count(&pool).await {
        Ok(count) if count > 0 => {
            info!(
                "Database already initialized with {} comments. Skipping scraping and inserts.",
                count
            );
            return;
        }
        Ok(_) => {
            info!("Database is empty. Proceeding to scrape and populate.");
        }
        Err(e) => {
            error!("Failed to check comments count: {}", e);
            return;
        }
    }

    // Load configuration using the `config` crate. The properties file is optional.
    let settings = Config::builder()
        .add_source(File::new(CONFIG_PATH, FileFormat::Ini).required(false))
        .build();

    match settings {
        Ok(settings) => {
            let url = settings
                .get_string("url")
                .unwrap_or_else(|_| DEFAULT_URL.to_string());

            let comments = get_comments(&url).await;
            info!("Parsed {} root comments", comments.len());

            let comments_batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);

            for batch in comments_batches.iter() {
                match insert_comments(&pool, batch).await {
                    Ok(n) => info!("Inserted {} comments into the database", n),
                    Err(e) => error!("Failed to insert comments: {}", e),
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to load config file '{}': {}. Using defaults.",
                CONFIG_PATH, e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::create_batches;

    #[test]
    fn test_create_batches_zero_size_returns_empty() {
        let items = vec![1, 2, 3, 4, 5];
        let batches: Vec<Vec<i32>> = create_batches(&items, 0);
        assert!(
            batches.is_empty(),
            "Expected empty vector when batch size is 0"
        );
    }

    #[test]
    fn test_create_batches_last_batch_not_full() {
        // 5 items with batch size 2 -> [[1,2],[3,4],[5]]
        let items = vec![1, 2, 3, 4, 5];
        let batches = create_batches(&items, 2);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![1, 2]);
        assert_eq!(batches[1], vec![3, 4]);
        assert_eq!(batches[2], vec![5]);
    }

    #[test]
    fn test_create_batches_exact_multiples() {
        // 6 items with batch size 2 -> [[1,2],[3,4],[5,6]]
        let items = vec![1, 2, 3, 4, 5, 6];
        let batches = create_batches(&items, 2);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![1, 2]);
        assert_eq!(batches[1], vec![3, 4]);
        assert_eq!(batches[2], vec![5, 6]);
    }
}
