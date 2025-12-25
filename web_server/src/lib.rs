pub mod api;
pub mod config;
pub mod scrape;
pub mod task_queue;
pub mod utils;
pub mod db;

#[derive(Debug, Default, Clone)]
pub struct CommentRecord {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    pub tags: Vec<String>,
}
