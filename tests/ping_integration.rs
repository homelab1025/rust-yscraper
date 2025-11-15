use axum::extract::Query;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test(flavor = "current_thread")]
async fn ping_happy_path_returns_ok_and_timestamp() {
    // Arrange: build the query parameters as axum's Query extractor expects
    let mut params = HashMap::new();
    params.insert("msg".to_string(), "hello".to_string());

    // Act: call the handler directly (no HTTP server)
    let response = rust_yscraper::api::ping(Query(params)).await;

    // Assert status
    assert!(response.is_ok());

    let pong = response.expect("Should be OK.");
    let body = &pong.msg;

    // Assert body starts with "hello " and contains a valid recent unix timestamp
    let prefix = "hello ";
    assert!(body.starts_with(prefix), "unexpected body: {body}");
    let ts_str = &body[prefix.len()..];
    let ts: u64 = ts_str.parse().expect("timestamp should be a u64");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.abs_diff(ts);
    assert!(
        diff <= 30,
        "timestamp too far from now: diff={diff}s, body={body}"
    );
}
