use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub msg: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ApiErrorCode {
    DatabaseError,
}
