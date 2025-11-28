use crate::api::ping::TimeProvider;
use crate::api::scrape_task::ScrapeTask;
use crate::db;
use crate::task_queue::TaskDedupQueue;
use axum::extract::FromRef;
use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn db::CommentsRepository>,
    pub time_provider: Arc<dyn TimeProvider>,
    pub http_client: Arc<Client>,
    pub task_queue: Arc<TaskDedupQueue<ScrapeTask>>,
}

#[derive(Clone)]
pub struct PingAppState {
    pub time_provider: Arc<dyn TimeProvider>,
}

#[derive(Clone)]
pub struct CommentsAppState {
    pub repo: Arc<dyn db::CommentsRepository>,
    // TODO: actually use this in the scraper
    pub http_client: Arc<Client>,
    pub task_queue: Arc<TaskDedupQueue<ScrapeTask>>,
}

impl FromRef<AppState> for PingAppState {
    fn from_ref(input: &AppState) -> Self {
        PingAppState {
            time_provider: input.time_provider.clone(),
        }
    }
}

impl FromRef<AppState> for CommentsAppState {
    fn from_ref(input: &AppState) -> Self {
        CommentsAppState {
            repo: input.repo.clone(),
            http_client: input.http_client.clone(),
            task_queue: input.task_queue.clone(),
        }
    }
}
