use axum::{
    routing::{get, post},
    Router,
};
use ::config::{Config, File, FileFormat};
use log::{error, info};
use rust_yscraper::api::app_state::AppState;
use rust_yscraper::api::comments::{list_comments, scrape_comments};
use rust_yscraper::api::ping::{ping, RealSystemTime};
use rust_yscraper::config::AppConfig;
use rust_yscraper::db::PgCommentsRepository;
use rust_yscraper::task_queue::TaskDedupQueue;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

const CONFIG_PATH: &str = "config.properties";

fn main() {
    // init logging first
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();
    info!("Starting server...");

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
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&cfg.db_url)
            .await
        {
            Ok(pool) => pool,
            Err(error) => {
                error!("Failed to initialize database: {}", error);
                return;
            }
        };

        let app_state = build_app_state(db_pool);

        // Build router
        let app = Router::new()
            .route("/ping", get(ping))
            .route("/scrape", post(scrape_comments))
            .route("/comments", get(list_comments))
            .with_state(app_state)
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );

        // Bind and serve
        let addr: SocketAddr = format!("0.0.0.0:{}", cfg.server_port).parse().unwrap();
        info!("Starting HTTP server at http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        if let Err(e) = axum::serve(listener, app).await {
            error!("Server error: {}", e);
        }
    });
}

fn build_app_state(db_pool: Pool<Postgres>) -> AppState {
    let queue = Arc::new(TaskDedupQueue::new(4));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("rust-yscraper/0.1 (+https://news.ycombinator.com)")
        .build()
        .unwrap();
    let http_client = Arc::new(client);

    let comments_repo = Arc::new(PgCommentsRepository::new(db_pool));
    let real_time_provider = Arc::new(RealSystemTime {});

    let app_state = AppState {
        repo: comments_repo,
        time_provider: real_time_provider,
        http_client,
        task_queue: queue,
    };
    app_state
}
