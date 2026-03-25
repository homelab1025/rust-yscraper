pub mod api;
pub mod background_scheduler;
pub mod config;
pub mod db;
pub mod middleware;
pub mod scrape;
pub mod scrape_task;
pub mod task_queue;
pub mod utils;

use axum::Router;
use axum::routing::{delete, get, patch, post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use api::app_state::AppState;
use api::auth::verify_totp;
use api::comments::{get_comment, list_comments, update_comment_state};
use api::links::{delete_link, list_links, refresh_link, scrape_link};
use api::ping::ping;
use middleware::auth::auth_middleware;

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/scrape", post(scrape_link))
        .route("/comments", get(list_comments))
        .route("/comments/{id}", get(get_comment))
        .route("/comments/{id}/state", patch(update_comment_state))
        .route("/links", get(list_links))
        .route("/links/{id}", delete(delete_link).patch(refresh_link))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let public = Router::new()
        .route("/ping", get(ping))
        .route("/auth/verify", post(verify_totp));

    Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state)
}

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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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
