use ::config::{Config, File, FileFormat};
use axum::{
    Router,
    routing::{get, post},
};
use log::{error, info};
use rust_yscraper::api::ping::RealSystemTime;
use rust_yscraper::config::AppConfig;
use rust_yscraper::db::SQLiteCommentsRepository;
use rust_yscraper::task_queue::TaskDedupQueueProcessor;
use rust_yscraper::{AppState, api};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

const CONFIG_PATH: &str = "config.properties";

async fn init_db(db_path: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Support both plain paths ("comments.db") and full SQLite URLs (e.g., "sqlite::memory:" or "sqlite:///file.db")
    let (db_url, should_manage_file): (String, bool) = if db_path.starts_with("sqlite:") {
        (db_path.to_string(), false)
    } else {
        (format!("sqlite://{}", db_path), true)
    };

    if should_manage_file
        && !Sqlite::database_exists(db_url.as_str())
            .await
            .unwrap_or(false)
    {
        info!("Initializing database at {}", db_path);
        Sqlite::create_database(db_url.as_str()).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Ensure foreign keys are enforced
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    // URLs table: stores HN item id, full URL, and date added (UTC)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY,
            url TEXT NOT NULL UNIQUE,
            date_added TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await?;

    // Comments table for fresh databases: includes the url_id foreign key
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author TEXT NOT NULL,
            date TEXT NOT NULL,
            text TEXT NOT NULL,
            url_id INTEGER NOT NULL,
            FOREIGN KEY (url_id) REFERENCES urls(id)
        )",
    )
    .execute(&pool)
    .await?;

    // Best-effort migration for existing DBs that may lack the url_id column
    // This will fail harmlessly if the column already exists
    let _ = sqlx::query("ALTER TABLE comments ADD COLUMN url_id INTEGER;")
        .execute(&pool)
        .await;

    Ok(pool)
}

fn main() {
    // init logging first
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    let cfg = AppConfig::from_config(
        &Config::builder()
            .add_source(File::new(CONFIG_PATH, FileFormat::Ini).required(false))
            .build()
            .expect("Failed to load config file"),
    );

    // Build a Tokio runtime and block on the async server startup.
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("failed to build Tokio runtime");

    tokio_rt.block_on(async move {
        // Initialize the database inside the async context
        let db_pool = match init_db(&cfg.db_path).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to initialize database: {}", e);
                return;
            }
        };

        let app_state = build_app_state(db_pool);

        // Build router
        let app = Router::new()
            .route("/ping", get(api::ping::ping))
            .route("/scrape", post(api::comments::scrape_comments))
            .route("/comments", get(api::comments::list_comments))
            .with_state(app_state)
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );

        // Bind and serve
        let addr: SocketAddr = format!("127.0.0.1:{}", cfg.server_port).parse().unwrap();
        info!("Starting HTTP server at http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        if let Err(e) = axum::serve(listener, app).await {
            error!("Server error: {}", e);
        }
    });
}

fn build_app_state(db_pool: Pool<Sqlite>) -> AppState {
    let queue = Arc::new(TaskDedupQueueProcessor::new(4));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("rust-yscraper/0.1 (+https://news.ycombinator.com)")
        .build()
        .unwrap();
    let http_client = Arc::new(client);

    let comments_repo = Arc::new(SQLiteCommentsRepository::new(db_pool));
    let real_time_provider = Arc::new(RealSystemTime {});

    let app_state = AppState {
        repo: comments_repo,
        time_provider: real_time_provider,
        http_client,
        task_queue: queue,
    };
    app_state
}
