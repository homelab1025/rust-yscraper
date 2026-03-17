pub mod app_state;
pub mod comments;
pub mod common;
pub mod links;
pub mod ping;

use crate::CommentState;
use crate::{SortBy, SortOrder};
use crate::api::comments::{CommentDto, CommentsPage, UpdateStateRequest};
use crate::api::common::{ApiError, ApiErrorCode};
use crate::api::links::{LinkDto, ScrapeRequest, ScrapeResponse, ScrapeState};
use crate::api::ping::PingResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::ping::ping,
        crate::api::comments::list_comments,
        crate::api::comments::get_comment,
        crate::api::comments::update_comment_state,
        crate::api::links::scrape_link,
        crate::api::links::list_links,
        crate::api::links::delete_link,
        crate::api::links::refresh_link,
    ),
    components(
        schemas(
            PingResponse,
            CommentDto,
            CommentsPage,
            UpdateStateRequest,
            CommentState,
            SortBy,
            SortOrder,
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
    ),
    info(
        license(identifier = "CC-BY-NC-ND-4.0")
    ),
)]
pub struct ApiDoc;
