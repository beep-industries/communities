use serde::Deserialize;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};

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
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
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
