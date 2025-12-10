use crate::domain::{
    channel::ports::ChannelRepository, friend::ports::FriendshipRepository,
    health::port::HealthRepository, server::ports::ServerRepository,
    server_member::ports::MemberRepository,
};

#[derive(Clone)]
pub struct Service<S, F, H, M, C>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F,
    pub(crate) health_repository: H,
    pub(crate) member_repository: M,
    pub(crate) channel_repository: C
}

impl<S, F, H, M,C> Service<S, F, H, M,C>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository

{
    pub fn new(
        server_repository: S,
        friendship_repository: F,
        health_repository: H,
        member_repository: M,
        channel_repository: C
    ) -> Self {
        Self {
            server_repository,
            friendship_repository,
            health_repository,
            member_repository,
            channel_repository
        }
    }
}
