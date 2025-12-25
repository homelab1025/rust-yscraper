use crate::api::ping::TimeProvider;
use crate::api::scrape_task::ScrapeTask;
use crate::db::CombinedRepository;
use crate::task_queue::TaskDedupQueue;
use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn CombinedRepository>,
    pub time_provider: Arc<dyn TimeProvider>,
    pub http_client: Arc<Client>,
    pub task_queue: Arc<TaskDedupQueue<ScrapeTask>>,
}
