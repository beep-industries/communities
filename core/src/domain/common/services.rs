use crate::domain::{
    channel::ports::ChannelRepository, friend::ports::FriendshipRepository,
    health::port::HealthRepository, role::ports::RoleRepository, server::ports::ServerRepository,
    server_member::ports::MemberRepository,
};

#[derive(Clone)]
pub struct Service<S, F, H, M, C, R>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F,
    pub(crate) health_repository: H,
    pub(crate) member_repository: M,
    pub(crate) channel_repository: C,
    pub(crate) role_repository: R,
}

impl<S, F, H, M, C, R> Service<S, F, H, M, C, R>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
{
    pub fn new(
        server_repository: S,
        friendship_repository: F,
        health_repository: H,
        member_repository: M,
        channel_repository: C,
        role_repository: R,
    ) -> Self {
        Self {
            server_repository,
            friendship_repository,
            health_repository,
            member_repository,
            channel_repository,
            role_repository,
        }
    }
}
