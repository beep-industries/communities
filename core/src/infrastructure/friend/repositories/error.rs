use thiserror::Error;

use crate::domain::friend::entities::UserId;

#[derive(Error, Debug, Clone)]
pub enum FriendshipError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Friend request not found")]
    FriendRequestNotFound,

    #[error("Friend request already exists")]
    FriendRequestAlreadyExists,

    #[error("No friend request found between {user1} and {user2}")]
    FailedToRemoveFriendRequest { user1: UserId, user2: UserId },

    #[error("Friendship already exists")]
    FriendshipAlreadyExists,

    #[error("Friendship not found")]
    FriendshipNotFound,
}

impl FriendshipError {
    pub fn error_code(&self) -> &'static str {
        match self {
            FriendshipError::FriendRequestAlreadyExists => "E_FRIEND_REQUEST_ALREADY_EXISTS",
            FriendshipError::FriendshipAlreadyExists => "E_FRIENDSHIP_ALREADY_EXISTS",
            _ => "E_UNKNOWN_FRIENDSHIP_ERROR",
        }
    }
}
