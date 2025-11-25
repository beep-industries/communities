use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use communities_core::{
    domain::common::CoreError, infrastructure::friend::repositories::error::FriendshipError,
};
use serde_json::json;

/// Unified error type for HTTP API responses
#[derive(Debug)]
pub enum ApiError {
    ServiceUnavailable(String),
    InternalServerError(String),
    AuthenticationError(String),
    NotFound(String),
    Forbidden(String),
    Unauthorized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed".to_string(),
            ),
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

impl From<FriendshipError> for ApiError {
    fn from(error: FriendshipError) -> Self {
        match error {
            FriendshipError::FriendRequestNotFound { user1: _, user2: _ } => {
                ApiError::NotFound(error.to_string())
            }
            FriendshipError::FriendRequestAlreadyExists { user1: _, user2: _ } => {
                ApiError::Forbidden(error.to_string())
            }
            FriendshipError::FailedToRemoveFriendRequest { user1: _, user2: _ } => {
                ApiError::Forbidden(error.to_string())
            }
            FriendshipError::FriendshipAlreadyExists { user1: _, user2: _ } => {
                ApiError::Forbidden(error.to_string())
            }
            FriendshipError::FriendshipNotFound { user1: _, user2: _ } => {
                ApiError::NotFound(error.to_string())
            }
            _ => ApiError::InternalServerError(error.to_string()),
        }
    }
}
