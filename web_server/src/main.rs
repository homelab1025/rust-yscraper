use ::config::{Config, File, FileFormat};
use axum::{
    Router,
    routing::{get, post},
};
use config::Environment;
use log::{error, info};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use web_server::api::ApiDoc;
use web_server::api::app_state::AppState;
use web_server::api::comments::{list_comments, scrape_comments};
use web_server::api::links::list_links;
use web_server::api::ping::{RealSystemTime, ping};
use web_server::config::AppConfig;
use web_server::db::postgresql::PgCommentsRepository;
use web_server::task_queue::TaskDedupQueue;

const CONFIG_PATH: &str = "config.toml";

fn main() {
    // init logging first
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();
    info!("Starting server...");

    let conf = Config::builder()
        .add_source(File::with_name(CONFIG_PATH).required(true))
        .add_source(Environment::with_prefix("YSCR").prefix_separator("_"))
        .build()
        .expect("Failed to load config file");

    let cfg = AppConfig::from_config(&conf).expect("Failed to create config structure.");

    let connection_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        &cfg.db_username, &cfg.db_password, &cfg.db_host, &cfg.db_port, &cfg.db_name
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
            .connect(&connection_url)
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
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .route("/ping", get(ping))
            .route("/scrape", post(scrape_comments))
            .route("/comments", get(list_comments))
            .route("/links", get(list_links))
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

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("web-server/0.1 (+https://news.ycombinator.com)")
        .build()
        .unwrap();
    let http_client = Arc::new(http_client);

    let comments_repo = Arc::new(PgCommentsRepository::new(db_pool));
    let real_time_provider = Arc::new(RealSystemTime {});

    AppState {
        repo: comments_repo,
        time_provider: real_time_provider,
        http_client,
        task_queue: queue,
    }
}
