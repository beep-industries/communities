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
