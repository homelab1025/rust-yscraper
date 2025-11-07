mod scrape;

use crate::scrape::get_comments;
use config::{Config, File, FileFormat};
use log::{error, info};
use rusqlite::{params, Connection};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::path::Path;

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

fn init_db<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author TEXT NOT NULL,
            date TEXT NOT NULL,
            text TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

fn insert_comments(conn: &Connection, comments: &Vec<CommentRecord>) -> rusqlite::Result<usize> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO comments (id, author, date, text) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut inserted = 0usize;
    for c in comments {
        let changes = stmt.execute(params![c.id, c.author, c.date, c.text])?;
        inserted += changes as usize; // OR IGNORE returns 0 when skipped due to PK conflict
    }
    Ok(inserted)
}

fn get_comment_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM comments", [], |row| row.get(0))
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

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    let db_path = "comments.db";
    info!("Initializing database at {}", db_path);
    let conn = match init_db(db_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return;
        }
    };

    match get_comment_count(&conn) {
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

            let comments = get_comments(&url);
            info!("Parsed {} root comments", comments.len());

            let comments_batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);

            comments_batches
                .iter()
                .for_each(|batch| match insert_comments(&conn, &batch) {
                    Ok(n) => info!("Inserted {} comments into the database", n),
                    Err(e) => error!("Failed to insert comments: {}", e),
                });
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
        assert!(batches.is_empty(), "Expected empty vector when batch size is 0");
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
