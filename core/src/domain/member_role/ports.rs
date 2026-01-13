use crate::domain::{common::CoreError, role::entities::RoleId, server_member::MemberId};
pub trait MemberRoleRepository: Send + Sync {
    fn assign(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<(), CoreError>>;
    fn unassign(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<(), CoreError>>;
}

pub trait MemberRoleService: Send + Sync {
    fn assign_member_to_role(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<(), CoreError>>;
    fn unassign_member_from_role(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<(), CoreError>>;
}

/// Mock implementation of MemberRoleRepository for testing
#[derive(Clone)]
pub struct MockMemberRoleRepository;

impl MemberRoleRepository for MockMemberRoleRepository {
    async fn assign(&self, _role_id: RoleId, _member_id: MemberId) -> Result<(), CoreError> {
        Ok(())
    }

    async fn unassign(&self, _role_id: RoleId, _member_id: MemberId) -> Result<(), CoreError> {
        Ok(())
    }
}
