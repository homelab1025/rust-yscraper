use crate::api::app_state::AppState;
use crate::api::common::{ApiError, ApiErrorCode};
use crate::db::links_repository::LinksRepository;
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone)]
pub struct LinksAppState {
    pub repo: Arc<dyn LinksRepository>,
}

impl FromRef<AppState> for LinksAppState {
    fn from_ref(input: &AppState) -> Self {
        LinksAppState {
            repo: input.repo.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LinkDto {
    pub id: i64,
    pub url: String,
    pub date_added: String,
}

/// Retrieve all links with their item IDs and added date.
#[utoipa::path(
    get,
    path = "/links",
    responses(
        (status = 200, description = "List of all links", body = [LinkDto]),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn list_links(
    State(state): State<LinksAppState>,
) -> Result<Json<Vec<LinkDto>>, (StatusCode, Json<ApiError>)> {
    let links = state.repo.list_links().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: ApiErrorCode::DatabaseError,
                msg: format!("Database error: {}", e),
            }),
        )
    })?;

    let dtos = links
        .into_iter()
        .map(|row| LinkDto {
            id: row.id,
            url: row.url,
            date_added: row.date_added.to_rfc3339(),
        })
        .collect();

    Ok(Json(dtos))
}
