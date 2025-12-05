use crate::domain::{
    friend::ports::FriendshipRepository, health::port::HealthRepository,
    server::ports::ServerRepository, server_member::ports::MemberRepository,
};

#[derive(Clone)]
pub struct Service<S, F, H, M>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F,
    pub(crate) health_repository: H,
    pub(crate) member_repository: M,
}

impl<S, F, H, M> Service<S, F, H, M>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
{
    pub fn new(
        server_repository: S,
        friendship_repository: F,
        health_repository: H,
        member_repository: M,
    ) -> Self {
        Self {
            server_repository,
            friendship_repository,
            health_repository,
            member_repository,
        }
    }
}
