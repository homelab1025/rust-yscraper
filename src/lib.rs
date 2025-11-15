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
    pub repo: std::sync::Arc<dyn db::CommentsRepository>,
}
