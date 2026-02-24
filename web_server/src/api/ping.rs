use crate::api::app_state::AppState;
use axum::extract::FromRef;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone)]
pub struct PingAppState {
    pub time_provider: Arc<dyn TimeProvider>,
}

impl FromRef<AppState> for PingAppState {
    fn from_ref(input: &AppState) -> Self {
        PingAppState {
            time_provider: input.time_provider.clone(),
        }
    }
}

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
#[into_params(parameter_in = Query)]
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
