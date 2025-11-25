use thiserror::Error;

use crate::domain::friend::entities::UserId;

#[derive(Error, Debug, Clone)]
pub enum FriendshipError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Friend request not found between {user1} and {user2}")]
    FriendRequestNotFound { user1: UserId, user2: UserId },

    #[error("Friend request already exists between {user1} and {user2}")]
    FriendRequestAlreadyExists { user1: UserId, user2: UserId },

    #[error("No friend request found between {user1} and {user2}")]
    FailedToRemoveFriendRequest { user1: UserId, user2: UserId },

    #[error("Friendship already exists between {user1} and {user2}")]
    FriendshipAlreadyExists { user1: UserId, user2: UserId },

    #[error("Friendship not found between {user1} and {user2}")]
    FriendshipNotFound { user1: UserId, user2: UserId },

    // == Mocked dataset Errors ==
    #[error("Mutex lock poisoned")]
    MutexLockPoisoned,
}