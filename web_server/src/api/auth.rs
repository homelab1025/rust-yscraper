use crate::api::app_state::AppState;
use crate::api::common::{ApiError, ApiErrorCode};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use log::warn;
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, Secret, TOTP};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyResponse {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/auth/verify",
    request_body = VerifyRequest,
    responses(
        (status = 200, description = "Authentication successful", body = VerifyResponse),
        (status = 401, description = "Invalid TOTP code", body = ApiError),
        (status = 500, description = "Server error", body = ApiError),
    ),
    tag = "web-server"
)]
pub async fn verify_totp(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<ApiError>)> {
    let secret_bytes = Secret::Encoded(state.config.totp_secret.clone())
        .to_bytes()
        .map_err(|e| {
            warn!("Invalid TOTP secret in config: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: ApiErrorCode::BadRequest,
                    msg: "Server misconfiguration".to_string(),
                }),
            )
        })?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        None,
        "yscraper".to_string(),
    )
    .map_err(|e| {
        warn!("Failed to build TOTP: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: ApiErrorCode::BadRequest,
                msg: "Server misconfiguration".to_string(),
            }),
        )
    })?;

    let valid = totp.check_current(&body.code).map_err(|e| {
        warn!("TOTP time error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: ApiErrorCode::BadRequest,
                msg: "Time error".to_string(),
            }),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: ApiErrorCode::BadRequest,
                msg: "Invalid code".to_string(),
            }),
        ));
    }

    let token = uuid::Uuid::new_v4().to_string();
    state.sessions.write().await.insert(token.clone());

    Ok(Json(VerifyResponse { token }))
}
