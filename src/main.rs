mod api;
mod scrape;
mod utils;

use axum::{
    Router,
    routing::{get, post},
};
use config::{Config, File, FileFormat};
use log::{error, info};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use std::net::SocketAddr;

// const DEFAULT_URL: &str = "https://news.ycombinator.com/item?id=45561428";
const CONFIG_PATH: &str = "config.properties";

#[derive(Debug, Default, Clone)]
pub struct CommentRecord {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db_pool: Pool<Sqlite>,
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

fn main() {
    // init logging first
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    // Load configuration using the `config` crate. The properties file is optional.
    // let settings = Config::builder()
    //     .add_source(File::new(CONFIG_PATH, FileFormat::Ini).required(false))
    //     .build();

    // Build a Tokio runtime and block on the async server startup.
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("failed to build Tokio runtime");

    tokio_rt.block_on(async move {
        // Initialize the database inside the async context
        let db_path = "comments.db";
        let db_pool = match init_db(db_path).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to initialize database: {}", e);
                return;
            }
        };

        let app_state = AppState { db_pool };

        // Build router
        let app = Router::new()
            .route("/ping", get(api::ping))
            .route("/scrape", post(api::scrape_hackernews))
            .with_state(app_state);

        // Bind and serve
        let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
        info!("Starting HTTP server at http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        if let Err(e) = axum::serve(listener, app).await {
            error!("Server error: {}", e);
        }
    });
}
