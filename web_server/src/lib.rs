pub mod api;
pub mod background_scheduler;
pub mod config;
pub mod db;
pub mod scrape;
pub mod scrape_task;
pub mod task_queue;
pub mod utils;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommentState {
    #[default]
    New = 0,
    Picked = 1,
    Discarded = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    Date,
    SubcommentCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl From<i32> for CommentState {
    fn from(v: i32) -> Self {
        match v {
            1 => Self::Picked,
            2 => Self::Discarded,
            _ => Self::New,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct CommentRecord {
    pub id: i64,
    pub author: String,
    pub date: String,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub state: CommentState,
    #[serde(default)]
    pub subcomment_count: i32,
}
