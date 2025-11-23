use axum::extract::{Query, State};
use axum::http::StatusCode;
use rust_yscraper::PingAppState;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTimeError};

struct MockTimeProvider {
    now_duration: Duration,
}

impl rust_yscraper::api::ping::TimeProvider for MockTimeProvider {
    fn now(&self) -> Result<Duration, SystemTimeError> {
        Ok(self.now_duration)
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ping_happy_path_returns_ok_and_timestamp() {
    // Arrange: build the query parameters as axum's Query extractor expects
    let mut params = HashMap::new();
    params.insert("msg".to_string(), "hello".to_string());

    let current_time = 10;
    let app_state = PingAppState {
        time_provider: Arc::new(MockTimeProvider {
            now_duration: Duration::from_secs(current_time),
        }),
    };

    // Act: call the handler directly (no HTTP server)
    let response = rust_yscraper::api::ping::ping(State(app_state), Query(params)).await;

    // Assert status
    assert!(response.is_ok());

    let pong = response.expect("Should be OK.");
    let body = &pong.msg;

    // Assert body starts with "hello" and contains a valid recent unix timestamp
    let prefix = "hello ";
    assert!(body.starts_with(prefix), "unexpected body: {body}");
    let ts_str = &body[prefix.len()..];
    let ts: u64 = ts_str.parse().expect("timestamp should be a u64");

    assert_eq!(ts, current_time, "unexpected timestamp: {ts}");
}

#[tokio::test(flavor = "current_thread")]
async fn ping_error_when_msg_missing() {
    // Arrange: no "msg" parameter provided
    let params = HashMap::new();
    let app_state = PingAppState {
        time_provider: Arc::new(MockTimeProvider {
            now_duration: Duration::from_secs(123),
        }),
    };

    // Act
    let response = rust_yscraper::api::ping::ping(State(app_state), Query(params)).await;

    // Assert: should be 400 with the expected error message
    assert!(response.is_err());
    let (status, body) = response.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "missing required query parameter: msg");
}
