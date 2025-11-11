use crate::domain::{friend::ports::FriendshipRepository, server::ports::ServerRepository};

pub struct Service<S, F>
where
    S: ServerRepository,
    F: FriendshipRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F
}

impl<S, F> Service<S, F>
where
    S: ServerRepository,
    F: FriendshipRepository,
{
    pub fn new(server_repository: S, friendship_repository: F) -> Self {
        Self { server_repository, friendship_repository }
    }
}
