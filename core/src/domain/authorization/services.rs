use beep_authz::SpiceDbObject;

use crate::{
    Service,
    domain::{
        authorization::ports::{AuthorizationRepository, AuthorizationService},
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::CoreError,
        friend::{entities::UserId, ports::FriendshipRepository},
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O, CM, MR, SI, A> AuthorizationService
    for Service<S, F, H, M, C, R, O, CM, MR, SI, A>
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
    A: AuthorizationRepository,
{
    fn check_authz(
        &self,
        user_id: UserId,
        permission: beep_authz::Permissions,
        resource: beep_authz::SpiceDbObject,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.authorization_repository.check_authz(
            SpiceDbObject::User(user_id.to_string()),
            permission,
            resource,
        )
    }
}
