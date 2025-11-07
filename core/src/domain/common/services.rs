use crate::domain::{friend::ports::{FriendRepository, FriendRequestRepository}, server::ports::ServerRepository};

pub struct Service<S, F, FR>
where
    S: ServerRepository,
    F: FriendRepository,
    FR: FriendRequestRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friend_repository: F,
    pub(crate) friend_request_repository: FR,
}

impl<S, F, FR> Service<S, F, FR>
where
    S: ServerRepository,
    F: FriendRepository,
    FR: FriendRequestRepository,
{
    pub fn new(server_repository: S, friend_repository: F, friend_request_repository: FR) -> Self {
        Self { server_repository, friend_repository, friend_request_repository }
    }
}
