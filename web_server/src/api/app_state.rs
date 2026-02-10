use crate::api::ping::TimeProvider;
use crate::config::AppConfig;
use crate::db::CombinedRepository;
use crate::scrape_task::ScrapeTask;
use crate::task_queue::TaskScheduler;
use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn CombinedRepository>,
    pub time_provider: Arc<dyn TimeProvider>,
    pub http_client: Arc<Client>,
    pub task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    pub config: AppConfig,
}
