pub mod app_state;
pub mod comments;
pub mod common;
pub mod ping;
pub mod scrape_task;

use crate::api::comments::{CommentDto, CommentsPage, ScrapeRequest, ScrapeResponse, ScrapeState};
use crate::api::common::{ApiError, ApiErrorCode};
use crate::api::ping::PingResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::ping::ping,
        crate::api::comments::list_comments,
        crate::api::comments::scrape_comments,
    ),
    components(
        schemas(
            PingResponse,
            CommentDto,
            CommentsPage,
            ScrapeRequest,
            ScrapeResponse,
            ScrapeState,
            ApiError,
            ApiErrorCode,
        )
    ),
    tags(
        (name = "web-server", description = "Hacker News Scraper API")
    )
)]
pub struct ApiDoc;