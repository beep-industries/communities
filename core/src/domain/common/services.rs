use crate::domain::{
    friend::ports::FriendshipRepository, health::port::HealthRepository,
    server::ports::ServerRepository,
};

pub struct Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F,
    pub(crate) health_repository: H,
}

impl<S, F, H> Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    pub fn new(server_repository: S, friendship_repository: F, health_repository: H) -> Self {
        Self {
            server_repository,
            friendship_repository,
            health_repository,
        }
    }
}
