use serde::Deserialize;
use thiserror::Error;

use crate::domain::{friend::entities::UserId, server::entities::ServerId};

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
    /// == Friends Errors ==
    #[error("Friend with id {id} not found")]
    FriendNotFound { id: String },

    #[error("Failed to insert friend with name {name}")]
    FailedToInsertFriend { name: String },

    #[error("Friend relationship already exists between {user1} and {user2}")]
    FriendshipAlreadyExists { user1: UserId, user2: UserId },

    #[error("Failed to remove friendship between {user1} and {user2}")]
    FailedToRemoveFriendship { user1: UserId, user2: UserId },
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