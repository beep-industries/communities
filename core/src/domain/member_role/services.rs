use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::CoreError,
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::{MemberRoleRepository, MemberRoleService},
        outbox::ports::OutboxRepository,
        role::{
            entities::{Role, RoleId},
            ports::RoleRepository,
        },
        server::ports::ServerRepository,
        server_member::{MemberId, MemberRepository, ServerMember},
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
    async fn assign_member_to_role(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        let role: Role = self.role_repository.find_by_id(&role_id).await?;
        let member: ServerMember = self.member_repository.find_by_id(member_id).await?;
        if member.server_id != role.server_id {
            return Err(CoreError::WrongSever);
        }
        let _ = self
            .member_role_repository
            .assign(role_id, member_id)
            .await?;
        Ok(())
    }

    async fn unassign_member_from_role(
        &self,
        role_id: crate::domain::role::entities::RoleId,
        member_id: crate::domain::server_member::MemberId,
    ) -> Result<(), CoreError> {
        Ok(self
            .member_role_repository
            .unassign(role_id, member_id)
            .await?)
    }
}
