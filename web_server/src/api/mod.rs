pub mod app_state;
pub mod comments;
pub mod common;
pub mod links;
pub mod ping;

use crate::api::comments::{CommentDto, CommentsPage, ScrapeRequest, ScrapeResponse, ScrapeState};
use crate::api::common::{ApiError, ApiErrorCode};
use crate::api::links::LinkDto;
use crate::api::ping::PingResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::ping::ping,
        crate::api::comments::list_comments,
        crate::api::comments::scrape_comments,
        crate::api::links::list_links,
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
            LinkDto,
        )
    ),
    tags(
        (name = "web-server", description = "Hacker News Scraper API")
    )
)]
pub struct ApiDoc;