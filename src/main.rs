mod scrape;

use crate::scrape::get_comments;
use config::{Config, File, FileFormat};
use log::{error, info};
use rusqlite::{params, Connection};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::path::Path;

const DEFAULT_URL: &str = "https://news.ycombinator.com/item?id=45561428";
const CONFIG_PATH: &str = "config.properties";

#[derive(Debug, Default)]
struct CommentRecord {
    id: i64,
    author: String,
    date: String,
    text: String,
    tags: Vec<String>,
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

fn insert_comments(conn: &Connection, comments: &[CommentRecord]) -> rusqlite::Result<usize> {
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

            match insert_comments(&conn, &comments) {
                Ok(n) => info!("Inserted {} comments into the database", n),
                Err(e) => error!("Failed to insert comments: {}", e),
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
