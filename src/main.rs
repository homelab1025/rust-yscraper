use config::{Config, File, FileFormat};
use log::{error, info, warn};
use rusqlite::{Connection, params};
use scraper::{Html, Selector};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::path::Path;
use std::time::Duration;

const DEFAULT_URL: &str = "https://news.ycombinator.com/item?id=45561428";
const CONFIG_PATH: &str = "config.properties";

#[derive(Debug)]
struct CommentRecord {
    id: i64,
    author: String,
    date: String,
    text: String,
}

fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("rust-yscraper/0.1 (+https://news.ycombinator.com)")
        .build()?;
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    let text = resp.text()?;
    Ok(text)
}

fn parse_root_comments(html: &str) -> Vec<CommentRecord> {
    let document = Html::parse_document(html);
    let tr_sel = Selector::parse("tr.athing.comtr").unwrap();
    let ind_img_sel = Selector::parse("td.ind img").unwrap();
    let author_sel = Selector::parse("a.hnuser").unwrap();
    let age_sel = Selector::parse("span.age").unwrap();
    let text_sel = Selector::parse(".comment").unwrap();
    let reply_sel = Selector::parse("a[href^=\"reply?\"]").unwrap();

    let mut out = Vec::new();

    for tr in document.select(&tr_sel) {
        // Determine indent: root comments have width=0 on the indent img
        let is_root = tr
            .select(&ind_img_sel)
            .next()
            .and_then(|img| img.value().attr("width"))
            .and_then(|w| w.parse::<i32>().ok())
            .map(|w| w == 0)
            .unwrap_or(true); // default to true if missing

        if !is_root {
            continue;
        }

        // Extract comment id from the reply link
        let id_opt = tr
            .select(&reply_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|href| {
                // href format: reply?id=<COMMENT_ID>&...
                href.split('?').nth(1).and_then(|qs| {
                    let mut out_id: Option<i64> = None;
                    for p in qs.split('&') {
                        if let Some(v) = p.strip_prefix("id=") {
                            if let Ok(n) = v.parse::<i64>() {
                                out_id = Some(n);
                                break;
                            }
                        }
                    }
                    out_id
                })
            });

        let id = match id_opt {
            Some(v) => v,
            None => {
                warn!("Skipping a root comment without a parsable reply id");
                continue;
            }
        };

        let author = tr
            .select(&author_sel)
            .next()
            .and_then(|a| Some(a.text().collect::<String>()))
            .unwrap_or_else(|| "".to_string());

        let date = tr
            .select(&age_sel)
            .next()
            .and_then(|span| span.value().attr("title").map(|s| s.to_string()))
            .or_else(|| tr.select(&age_sel).next().map(|s| s.text().collect()))
            .unwrap_or_else(|| "".to_string());

        let text = tr
            .select(&text_sel)
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_else(|| "".to_string());

        if author.is_empty() && text.trim().is_empty() {
            continue;
        }

        out.push(CommentRecord {
            id,
            author,
            date,
            text,
        });
    }

    out
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

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    // Load configuration using the `config` crate. The properties file is optional.
    let settings = Config::builder()
        .add_source(File::new(CONFIG_PATH, FileFormat::Ini).required(false))
        .build();

    match settings {
        Ok(settings) => {
            let url = settings
                .get_string("url")
                .unwrap_or_else(|_| DEFAULT_URL.to_string());

            info!("Fetching URL: {}", url);
            let html = match fetch_html(&url) {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to fetch '{}': {}", url, e);
                    std::process::exit(1);
                }
            };

            info!("Parsing root comments...");
            let comments = parse_root_comments(&html);
            info!("Parsed {} root comments", comments.len());

            let db_path = "comments.db";
            info!("Initializing database at {}", db_path);
            match init_db(db_path) {
                Ok(conn) => match insert_comments(&conn, &comments) {
                    Ok(n) => info!("Inserted {} comments into the database", n),
                    Err(e) => error!("Failed to insert comments: {}", e),
                },
                Err(e) => error!("Failed to initialize database: {}", e),
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
