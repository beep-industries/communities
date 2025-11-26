use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use communities_core::{
    domain::common::CoreError, infrastructure::friend::repositories::error::FriendshipError,
};
use serde::Serialize;
use thiserror::Error;

/// Unified error type for HTTP API responses
#[derive(Debug, Error, Clone)]
pub enum ApiError {
    #[error("Service is unavailable: {msg}")]
    ServiceUnavailable { msg: String },
    #[error("Internal server error")]
    InternalServerError,
    #[error("Startup error: {msg}")]
    StartupError { msg: String },
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found: {msg}")]
    NotFound { msg: String },
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::StartupError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
        }
    }
}

impl Into<ErrorBody> for ApiError {
    fn into(self) -> ErrorBody {
        ErrorBody {
            message: self.to_string(),
            status: self.status_code().as_u16(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status_code(), Json::<ErrorBody>(self.into())).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::Unhealthy => ApiError::ServiceUnavailable {
                msg: "Service is unhealthy".to_string(),
            },
            _ => ApiError::InternalServerError,
        }
    }
}

impl From<FriendshipError> for ApiError {
    fn from(error: FriendshipError) -> Self {
        match error {
            FriendshipError::FriendRequestAlreadyExists { user1: _, user2: _ } => {
                ApiError::Forbidden
            }
            FriendshipError::FailedToRemoveFriendRequest { user1: _, user2: _ } => {
                ApiError::Forbidden
            }
            FriendshipError::FriendshipAlreadyExists { user1: _, user2: _ } => ApiError::Forbidden,
            FriendshipError::FriendshipNotFound { user1: _, user2: _ } => ApiError::NotFound {
                msg: error.to_string(),
            },
            _ => ApiError::InternalServerError,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub message: String,
    pub status: u16,
}
