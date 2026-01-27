use crate::{
    Service,
    domain::{
        authorization::ports::AuthorizationRepository,
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::{
            entities::{AssignMemberRole, MemberRole, UnassignMemberRole},
            ports::{MemberRoleRepository, MemberRoleService},
        },
        outbox::ports::OutboxRepository,
        role::{
            entities::{Role, RoleId},
            ports::RoleRepository,
        },
        server::ports::ServerRepository,
        server_invitation::ports::ServerInvitationRepository,
        server_member::{MemberId, MemberRepository, ServerMember},
        server_pictures::ServerPicturesRepository,
        user::port::UserRepository,
    },
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> MemberRoleService
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
    async fn assign_member_to_role(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> Result<MemberRole, CoreError> {
        let role: Role = self.role_repository.find_by_id(&role_id).await?;
        let member: ServerMember = self.member_repository.find_by_id(member_id).await?;
        if member.server_id != role.server_id {
            return Err(CoreError::BadRoleMemberAssignation);
        }
        let member_role = self
            .member_role_repository
            .assign(AssignMemberRole { role_id, member_id })
            .await?;
        Ok(member_role)
    }

    async fn unassign_member_from_role(
        &self,
        role_id: crate::domain::role::entities::RoleId,
        member_id: crate::domain::server_member::MemberId,
    ) -> Result<(), CoreError> {
        self.member_role_repository
            .unassign(UnassignMemberRole { role_id, member_id })
            .await?;
        Ok(())
    }

    async fn list_members_by_role(
        &self,
        role_id: &RoleId,
        pagination: &GetPaginated,
    ) -> Result<(Vec<ServerMember>, TotalPaginatedElements), CoreError> {
        self.member_role_repository
            .list_members_by_role(role_id, pagination)
            .await
    }

    async fn list_roles_by_user_and_server(
        &self,
        user_id: crate::domain::friend::entities::UserId,
        server_id: crate::domain::server::entities::ServerId,
    ) -> Result<Vec<Role>, CoreError> {
        self.member_role_repository
            .list_roles_by_user_and_server(user_id, server_id)
            .await
    }
}
