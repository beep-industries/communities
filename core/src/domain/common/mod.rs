use serde::Deserialize;
use thiserror::Error;

use crate::domain::server::entities::ServerId;

pub mod services;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Server with id {id} not found")]
    ServerNotFound { id: ServerId },

    #[error("Failed to insert server with name {name}")]
    FailedToInsertServer { name: String },

    #[error("Failed to manipulate with friendship data")]
    FriendshipDataError,
    
    #[error("Health check failed")]
    Unhealthy,
}

#[derive(Debug, Deserialize)]
pub struct GetPaginated {
    pub page: u32,
    pub limit: u32,
}

impl Default for GetPaginated {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20
        }
    }
}