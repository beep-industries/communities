use crate::domain::{
    channel::ports::ChannelRepository, channel_member::ports::ChannelMemberRepository,
    friend::ports::FriendshipRepository, health::port::HealthRepository,
    member_role::ports::MemberRoleRepository, outbox::ports::OutboxRepository,
    role::ports::RoleRepository, server::ports::ServerRepository,
    server_member::ports::MemberRepository,
    server_invitation::ports::ServerInvitationRepository,
};

#[derive(Clone)]
pub struct Service<S, F, H, M, C, R, O, CM, MR, SI>
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
    SI: ServerInvitationRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F,
    pub(crate) health_repository: H,
    pub(crate) member_repository: M,
    pub(crate) channel_repository: C,
    pub(crate) role_repository: R,
    pub(crate) outbox_repository: O,
    pub(crate) channel_member_repository: CM,
    pub(crate) member_role_repository: MR,
    pub(crate) server_invitation_repository: SI,
}

impl<S, F, H, M, C, R, O, CM, MR, SI> Service<S, F, H, M, C, R, O, CM, MR, SI>
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
    SI: ServerInvitationRepository,
{
    pub fn new(
        server_repository: S,
        friendship_repository: F,
        health_repository: H,
        member_repository: M,
        channel_repository: C,
        role_repository: R,
        outbox_repository: O,
        channel_member_repository: CM,
        member_role_repository: MR,
        server_invitation_repository: SI,
    ) -> Self {
        Self {
            server_repository,
            friendship_repository,
            health_repository,
            member_repository,
            channel_repository,
            role_repository,
            outbox_repository,
            member_role_repository,
            channel_member_repository,
            server_invitation_repository,
        }
    }
}
