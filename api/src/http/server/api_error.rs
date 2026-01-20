use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use communities_core::{
    domain::common::CoreError,
    infrastructure::{
        friend::repositories::error::FriendshipError, user::repositories::error::UserError,
    },
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
    NotFound { error_code: Option<String> },
    #[error("Bad request: {msg}")]
    BadRequest {
        msg: String,
        error_code: Option<String>,
    },
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
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
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
            ApiError::NotFound { error_code } => ErrorBody {
                message: message,
                error_code: error_code,
                status: status,
            },
            ApiError::BadRequest { msg, error_code } => ErrorBody {
                message: msg,
                error_code: error_code,
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
            CoreError::ServerNotFound { .. } => ApiError::NotFound { error_code: None },
            CoreError::InvalidServerName => ApiError::BadRequest {
                msg: "Server name cannot be empty".to_string(),
                error_code: None,
            },
            CoreError::MemberNotFound { .. } => ApiError::NotFound { error_code: None },
            CoreError::MemberAlreadyExists { .. } => ApiError::Conflict {
                error_code: "MEMBER_ALREADY_EXISTS".to_string(),
            },
            CoreError::InvalidMemberNickname => ApiError::BadRequest {
                msg: "Invalid member nickname: cannot be empty or whitespace".to_string(),
                error_code: None,
            },
            CoreError::ChannelNotFound { .. } => ApiError::NotFound { error_code: None },
            CoreError::ChannelPayloadError { msg, .. } => ApiError::BadRequest {
                msg,
                error_code: None,
            },
            CoreError::Forbidden => ApiError::Forbidden,
            _ => ApiError::InternalServerError,
        }
    }
}

impl From<FriendshipError> for ApiError {
    fn from(error: FriendshipError) -> Self {
        match error {
            FriendshipError::CannotFriendYourself => ApiError::BadRequest {
                msg: "Cannot send a friend request to yourself".to_string(),
                error_code: Some(error.error_code().to_string()),
            },
            FriendshipError::FriendRequestNotFound => ApiError::NotFound { error_code: None },
            FriendshipError::FriendRequestAlreadyExists => ApiError::Conflict {
                error_code: error.error_code().to_string(),
            },
            FriendshipError::FailedToRemoveFriendRequest => ApiError::Forbidden,
            FriendshipError::FriendshipAlreadyExists => ApiError::Conflict {
                error_code: error.error_code().to_string(),
            },
            FriendshipError::FriendshipNotFound => ApiError::NotFound { error_code: None },
            FriendshipError::UserNotFound => ApiError::NotFound {
                error_code: Some(error.error_code().to_string()),
            },
            _ => ApiError::InternalServerError,
        }
    }
}

impl From<UserError> for ApiError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::UserNotFound => ApiError::NotFound {
                error_code: error.error_code().to_string().into(),
            },
            _ => ApiError::InternalServerError,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub message: String,
    pub error_code: Option<String>,
    pub status: u16,
}
