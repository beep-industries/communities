use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use communities_core::{
    domain::common::CoreError, infrastructure::friend::repositories::error::FriendshipError,
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use communities_core::domain::common::CoreError;

/// Unified error type for HTTP API responses
#[derive(Debug, Error, Clone)]
pub enum ApiError {
    #[error("Service is unavailable: {msg}")]
    ServiceUnavailable { msg: String },
    #[error("Internal server error: {msg}")]
    InternalServerError { msg: String },
    #[error("Startup error: {msg}")]
    StartupError { msg: String },
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Forbidden: {0}")]
    Forbidden(String),
}

impl ApiError {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            ApiError::StartupError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

impl Into<ErrorBody> for ApiError {
    fn into(self) -> ErrorBody {
        ErrorBody {
            message: self.to_string(),
            status: self.to_status_code().as_u16(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Json::<ErrorBody>(self.into()).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::Unhealthy => ApiError::ServiceUnavailable {
                msg: "Service is unhealthy".to_string(),
            },
            _ => ApiError::InternalServerError {
                msg: error.to_string(),
            },
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
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub message: String,
    pub status: u16,
}
