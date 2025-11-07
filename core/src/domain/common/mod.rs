use thiserror::Error;

use crate::domain::server::entities::ServerId;

pub mod services;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Server with id {id} not found")]
    ServerNotFound { id: ServerId },

    #[error("Failed to insert server with name {name}")]
    FailedToInsertServer { name: String },
}
