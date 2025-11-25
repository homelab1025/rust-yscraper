use axum::extract::FromRef;
use reqwest::Client;
use std::sync::Arc;

pub mod api;
pub mod config;
pub mod db;
pub mod scrape;
pub mod utils;

#[derive(Debug, Default, Clone)]
pub struct CommentRecord {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn db::CommentsRepository>,
    pub time_provider: Arc<dyn api::ping::TimeProvider>,
    pub http_client: Arc<Client>,
}

#[derive(Clone)]
pub struct PingAppState {
    pub time_provider: Arc<dyn api::ping::TimeProvider>,
}

#[derive(Clone)]
pub struct CommentsAppState {
    pub repo: Arc<dyn db::CommentsRepository>,
    // TODO: actually use this in the scraper
    pub http_client: Arc<Client>,
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
        }
    }
}
