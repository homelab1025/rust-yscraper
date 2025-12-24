use crate::api::app_state::PingAppState;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use utoipa::{IntoParams, ToSchema};

/// Abstraction around system time for easy testing.
pub trait TimeProvider: Send + Sync {
    fn now(&self) -> Result<Duration, SystemTimeError>;
}

/// Real implementation of `TimeProvider` using `SystemTime`.
#[derive(Default)]
pub struct RealSystemTime {}

impl TimeProvider for RealSystemTime {
    fn now(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(UNIX_EPOCH)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PingResponse {
    pub msg: String,
}

#[derive(serde::Deserialize, IntoParams)]
pub struct PingParams {
    /// Message to echo back
    pub msg: Option<String>,
}

/// Health check handler: echoes back the provided `msg` with the current Unix timestamp
#[utoipa::path(
    get,
    path = "/ping",
    params(PingParams),
    responses(
        (status = 200, description = "Ping successful", body = PingResponse),
        (status = 400, description = "Missing required query parameter: msg", body = String)
    )
)]
#[axum::debug_handler]
pub async fn ping(
    State(state): State<PingAppState>,
    Query(params): Query<PingParams>,
) -> Result<Json<PingResponse>, (axum::http::StatusCode, String)> {
    let time_provider = state.time_provider;
    match params.msg.as_ref().filter(|m| !m.is_empty()) {
        Some(msg) => {
            let ts = time_provider.now().map(|d| d.as_secs()).unwrap_or(0);
            let body = format!("{} {}", msg, ts);
            Ok(Json(PingResponse { msg: body }))
        }
        None => Err((
            axum::http::StatusCode::BAD_REQUEST,
            "missing required query parameter: msg".to_string(),
        )),
    }
}
