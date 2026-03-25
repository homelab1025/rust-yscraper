use crate::api::app_state::AppState;
use crate::api::common::{ApiError, ApiErrorCode};
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned);

    match token {
        Some(t) if state.sessions.read().await.contains(&t) => next.run(req).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: ApiErrorCode::BadRequest,
                msg: "Unauthorized".to_string(),
            }),
        )
            .into_response(),
    }
}
