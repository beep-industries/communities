use serde::Deserialize;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};

use crate::domain::channel::entities::{ChannelError, ChannelId};
use crate::domain::friend::entities::UserId;
use crate::domain::server::entities::ServerId;

pub mod services;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Service is currently unavailable")]
    ServiceUnavailable(String),

    #[error("Server with id {id} not found")]
    ServerNotFound { id: ServerId },

    #[error("Failed to insert server with name {name}")]
    FailedToInsertServer { name: String },

    #[error("Server name cannot be empty")]
    InvalidServerName,

    #[error("Failed to manipulate with friendship data")]
    FriendshipDataError,

    #[error("Health check failed")]
    Unhealthy,

    #[error("An unknown error occurred: {message}")]
    UnknownError { message: String },

    #[error("Database error: {msg}")]
    DatabaseError { msg: String },

    /// Serialization error occurred when converting event to JSON
    #[error("Serialization error: {msg}")]
    SerializationError { msg: String },

    #[error("Member already exists for server {server_id} and user {user_id}")]
    MemberAlreadyExists {
        server_id: ServerId,
        user_id: UserId,
    },

    #[error("Member not found for server {server_id} and user {user_id}")]
    MemberNotFound {
        server_id: ServerId,
        user_id: UserId,
    },

    #[error("Invalid member nickname: cannot be empty or whitespace")]
    InvalidMemberNickname,

    #[error("Failed to insert member for server {server_id} and user {user_id}")]
    FailedToInsertMember {
        server_id: ServerId,
        user_id: UserId,
    },

    #[error("Channel with id {id} not found")]
    ChannelNotFound { id: ChannelId },

    #[error("Failed to create resource: {msg}")]
    CreationFailure { msg: String },
}

impl From<ChannelError> for CoreError {
    fn from(value: ChannelError) -> Self {
        match value {
            ChannelError::ChannelNameTooLong | ChannelError::WrongChannelType => {
                Self::CreationFailure {
                    msg: value.to_string(),
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetPaginated {
    pub page: u32,
    pub limit: u32,
}

impl Default for GetPaginated {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

pub type TotalPaginatedElements = u64;
