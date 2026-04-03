use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

const GIT_HASH: &str = match option_env!("GIT_HASH") {
    Some(v) => v,
    None => "unknown",
};
const GIT_COMMITTED_AT: &str = match option_env!("GIT_COMMITTED_AT") {
    Some(v) => v,
    None => "unknown",
};

#[derive(Debug, Serialize, ToSchema)]
pub struct InfoResponse {
    pub git_hash: String,
    pub committed_at: String,
}

/// Returns build info such as the deployed git commit hash and commit datetime.
#[utoipa::path(
    get,
    path = "/info",
    responses(
        (status = 200, description = "Build info", body = InfoResponse),
    )
)]
pub async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        git_hash: GIT_HASH[..GIT_HASH.len().min(10)].to_string(),
        committed_at: GIT_COMMITTED_AT.to_string(),
    })
}
