use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::CoreError,
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
        server_member::{MemberId, MemberRepository, ServerMember},
        server_invitation::ports::ServerInvitationRepository,
    },
};

impl<S, F, H, M, C, R, O, CM, MR, SI> MemberRoleService for Service<S, F, H, M, C, R, O, CM, MR, SI>
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
}
