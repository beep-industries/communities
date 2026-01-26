use crate::domain::{
    authorization::ports::AuthorizationRepository, channel::ports::ChannelRepository,
    channel_member::ports::ChannelMemberRepository, friend::ports::FriendshipRepository,
    health::port::HealthRepository, member_role::ports::MemberRoleRepository,
    outbox::ports::OutboxRepository, role::ports::RoleRepository, server::ports::ServerRepository,
    server_invitation::ports::ServerInvitationRepository, server_member::ports::MemberRepository,
    server_pictures::ServerPicturesRepository, user::port::UserRepository,
};

#[derive(Clone, Debug)]
pub struct Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
    SC: ServerPicturesRepository,
{
    pub(crate) server_repository: S,
    pub(crate) friendship_repository: F,
    pub(crate) user_repository: U,
    pub(crate) health_repository: H,
    pub(crate) member_repository: M,
    pub(crate) channel_repository: C,
    pub(crate) role_repository: R,
    pub(crate) outbox_repository: O,
    pub(crate) channel_member_repository: CM,
    pub(crate) member_role_repository: MR,
    pub(crate) server_invitation_repository: SI,
    pub(crate) authorization_repository: A,
    pub(crate) server_pictures_repository: SC,
}

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
    SC: ServerPicturesRepository,
{
    pub fn new(
        server_repository: S,
        friendship_repository: F,
        user_repository: U,
        health_repository: H,
        member_repository: M,
        channel_repository: C,
        role_repository: R,
        outbox_repository: O,
        channel_member_repository: CM,
        member_role_repository: MR,
        server_invitation_repository: SI,
        authorization_repository: A,
        server_pictures_repository: SC,
    ) -> Self {
        Self {
            server_repository,
            friendship_repository,
            user_repository,
            health_repository,
            member_repository,
            channel_repository,
            role_repository,
            outbox_repository,
            member_role_repository,
            channel_member_repository,
            server_invitation_repository,
            authorization_repository,
            server_pictures_repository,
        }
    }
}
