// Shared helpers used by multiple integration test binaries; not all items are
// used in every binary, so suppress the dead-code lint for this module.
#![allow(dead_code)]

use async_trait::async_trait;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use testcontainers_modules::postgres::Postgres;
use tokio::sync::Mutex;
use tokio::sync::mpsc::error::TrySendError;
use web_server::api::app_state::AppState;
use web_server::api::ping::RealSystemTime;
use web_server::config::AppConfig;
use web_server::db::postgresql::PgCommentsRepository;
use web_server::scrape::{CommentScraper, ScrapeError, ScrapeResult};
use web_server::scrape_task::ScrapeTask;
use web_server::task_queue::TaskScheduler;

pub struct StubScheduler;

#[async_trait]
impl TaskScheduler<ScrapeTask> for StubScheduler {
    async fn schedule(&self, _task: ScrapeTask) -> Result<bool, TrySendError<ScrapeTask>> {
        Ok(true)
    }

    async fn shutdown(&self) {}
}

pub struct RecordingScheduler {
    pub scheduled: Arc<Mutex<Vec<(i64, String)>>>,
    pub outcome: bool,
}

#[async_trait]
impl TaskScheduler<ScrapeTask> for RecordingScheduler {
    async fn schedule(&self, task: ScrapeTask) -> Result<bool, TrySendError<ScrapeTask>> {
        self.scheduled
            .lock()
            .await
            .push((task.url_id(), task.url().to_string()));
        Ok(self.outcome)
    }

    async fn shutdown(&self) {}
}

pub struct NoOpScraper;

#[async_trait]
impl CommentScraper for NoOpScraper {
    async fn get_comments(&self, _url: &str) -> Result<ScrapeResult, ScrapeError> {
        Ok(ScrapeResult {
            comments: vec![],
            thread_month: None,
            thread_year: None,
        })
    }
}

pub fn make_test_app_state(pool: PgPool) -> AppState {
    make_test_app_state_with_scheduler(pool, Arc::new(StubScheduler))
}

pub fn make_test_app_state_with_scheduler(
    pool: PgPool,
    scheduler: Arc<dyn TaskScheduler<ScrapeTask>>,
) -> AppState {
    AppState {
        repo: Arc::new(PgCommentsRepository::new(pool)),
        time_provider: Arc::new(RealSystemTime {}),
        task_queue: scheduler,
        scraper: Arc::new(NoOpScraper),
        config: AppConfig {
            server_port: 3000,
            db_username: "u".to_string(),
            db_password: "p".to_string(),
            db_name: "n".to_string(),
            db_host: "h".to_string(),
            db_port: 5432,
            default_days_limit: 7,
            default_frequency_hours: 24,
        },
    }
}

pub async fn setup_db() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .with_user("postgres")
        .with_password("postgres")
        .with_db_name("postgres")
        .with_network("bridge")
        .with_log_consumer(|log: &testcontainers::core::logs::LogFrame| {
            print!("{}", String::from_utf8_lossy(log.bytes()));
        })
        .start()
        .await
        .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    // Get the internal bridge IP of the Postgres container for Liquibase to connect to.
    let postgres_ip = container.get_bridge_ip_address().await.unwrap();
    let jdbc_url = format!("jdbc:postgresql://{}:5432/postgres", postgres_ip);

    let project_root = env::current_dir().unwrap().parent().unwrap().to_path_buf();
    let db_path = project_root.join("db");
    let db_path_str = db_path.to_str().unwrap();

    let liquibase = GenericImage::new("liquibase/liquibase", "4.23")
        .with_network("bridge")
        .with_mount(testcontainers::core::Mount::bind_mount(
            db_path_str,
            "/liquibase/db",
        ))
        .with_log_consumer(|log: &testcontainers::core::logs::LogFrame| {
            print!("{}", String::from_utf8_lossy(log.bytes()));
        })
        .with_cmd([
            "--changelog-file=db/changelog/db.changelog-master.yaml",
            &format!("--url={}", jdbc_url),
            "--username=postgres",
            "--password=postgres",
            "update",
        ])
        .with_ready_conditions(vec![WaitFor::message_on_stdout("UPDATE SUMMARY")]);

    let _liquibase_container = AsyncRunner::start(liquibase).await.unwrap();

    let conn_str = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
    let pool = PgPool::connect(&conn_str)
        .await
        .expect("Failed to connect to Postgres");

    (pool, container)
}
