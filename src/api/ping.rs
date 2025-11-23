use axum::{extract::{Query, State}, Json};
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use crate::PingAppState;

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

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub msg: String,
}

/// Health check handler: echoes back the provided `msg` with the current Unix timestamp
#[axum::debug_handler]
pub async fn ping(
    State(state): State<PingAppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PingResponse>, (axum::http::StatusCode, String)> {
    let time_provider = state.time_provider;
    match params.get("msg").filter(|m| !m.is_empty()) {
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
