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
    #[error("Not found")]
    NotFound,
    #[error("Conflict")]
    Conflict { error_code: String },
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
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
        }
    }
}

impl Into<ErrorBody> for ApiError {
    fn into(self) -> ErrorBody {
        let status = self.status_code().as_u16();
        let message = self.to_string();
        match self {
            ApiError::Conflict { error_code } => ErrorBody {
                message: message,
                error_code: Some(error_code),
                status: status,
            },
            _ => ErrorBody {
                message: message,
                error_code: None,
                status: status,
            },
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
            FriendshipError::FriendRequestAlreadyExists => ApiError::Conflict {
                error_code: error.error_code().to_string(),
            },
            FriendshipError::FailedToRemoveFriendRequest { user1: _, user2: _ } => {
                ApiError::Forbidden
            }
            FriendshipError::FriendshipAlreadyExists => ApiError::Conflict {
                error_code: error.error_code().to_string(),
            },
            FriendshipError::FriendshipNotFound => ApiError::NotFound,
            _ => ApiError::InternalServerError,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub message: String,
    pub error_code: Option<String>,
    pub status: u16,
}
