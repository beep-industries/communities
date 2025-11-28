use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum FriendshipError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Friend request not found")]
    FriendRequestNotFound,

    #[error("Friend request already exists")]
    FriendRequestAlreadyExists,

    #[error("No friend request found")]
    FailedToRemoveFriendRequest,

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
