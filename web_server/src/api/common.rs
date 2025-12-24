use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub msg: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ApiErrorCode {
    DatabaseError,
    BadRequest,
    SchedulingError,
}
