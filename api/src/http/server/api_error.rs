use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use core::domain::common::CoreError;
use serde_json::json;

/// Unified error type for HTTP API responses
#[derive(Debug)]
pub enum ApiError {
    ServiceUnavailable(String),
    InternalServerError(String),
    AuthenticationError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::Unhealthy => {
                ApiError::ServiceUnavailable("Service is unhealthy".to_string())
            }
            _ => ApiError::InternalServerError(error.to_string()),
        }
    }
}
