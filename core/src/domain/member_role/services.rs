use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::{MemberRoleRepository, MemberRoleService},
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O, CM, MR> MemberRoleService for Service<S, F, H, M, C, R, O, CM, MR>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
{
}
