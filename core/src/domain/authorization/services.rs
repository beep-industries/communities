use beep_auth::User;
use beep_authz::{Permissions, SpiceDbObject};

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
        server::{entities::ServerId, ports::ServerRepository},
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
        server_pictures::ServerPicturesRepository,
        user::port::UserRepository,
    },
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> AuthorizationService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
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

    fn can_manage_roles_in_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            beep_authz::Permissions::ManageRoles,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }

    fn can_manage_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            beep_authz::Permissions::ManageServer,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }

    fn can_manage_channels_in_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            Permissions::ManageChannels,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }

    fn can_view_channels_in_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            beep_authz::Permissions::ViewChannels,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }

    fn can_change_nickname(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            beep_authz::Permissions::ChangeNickname,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }

    fn can_update_nickname(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            beep_authz::Permissions::ManageNicknames,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }

    fn can_create_invitation(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>> {
        self.check_authz(
            user_id,
            beep_authz::Permissions::CreateInvitation,
            SpiceDbObject::Server(server_id.to_string()),
        )
    }
}
