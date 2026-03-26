use ::config::Config;
use config::{Environment, File, FileFormat};
use log::{error, info};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use sqlx::postgres::PgPoolOptions;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use web_server::api::ApiDoc;
use web_server::api::app_state::AppState;
use web_server::api::ping::RealSystemTime;
use web_server::background_scheduler::BackgroundScheduler;
use web_server::config::AppConfig;
use web_server::db::CombinedRepository;
use web_server::db::postgresql::PgCommentsRepository;
use web_server::scrape::{CommentScraper, DefaultScraper, ReqwestHttpClient};
use web_server::scrape_task::ScrapeTask;
use web_server::task_queue::{TaskDedupQueue, TaskScheduler};

const CONFIG_PATH: &str = "config.toml";

fn main() {
    // init logging first
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();
    info!("Starting server...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| CONFIG_PATH.to_string());

    let conf = Config::builder()
        .add_source(File::new(&config_path, FileFormat::Toml))
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

        let comments_repo = Arc::new(PgCommentsRepository::new(db_pool.clone()));
        let task_queue = Arc::new(TaskDedupQueue::new(4));
        let scraper: Arc<dyn CommentScraper> =
            Arc::new(DefaultScraper::new(Arc::new(ReqwestHttpClient::new())));

        // Start background scheduler
        start_background_scheduler(comments_repo.clone(), task_queue.clone(), scraper.clone())
            .await;

        let app_state = build_app_state(comments_repo, task_queue, scraper, cfg.clone());

        // Build router
        let app = web_server::build_router(app_state)
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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

fn build_app_state(
    comments_repo: Arc<dyn CombinedRepository>,
    task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    scraper: Arc<dyn CommentScraper>,
    config: AppConfig,
) -> AppState {
    let real_time_provider = Arc::new(RealSystemTime {});

    AppState {
        repo: comments_repo,
        time_provider: real_time_provider,
        task_queue,
        scraper,
        config,
    }
}

async fn start_background_scheduler(
    repo: Arc<dyn CombinedRepository>,
    task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    scraper: Arc<dyn CommentScraper>,
) {
    let bg_scheduler = BackgroundScheduler::new(
        repo.clone(),
        task_queue.clone(),
        scraper,
        Duration::from_secs(60), // Check every minute
    );

    tokio::spawn(async move {
        bg_scheduler.run().await;
    });
}
