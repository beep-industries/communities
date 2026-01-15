use std::sync::{Arc, Mutex};

use crate::domain::{
    common::CoreError,
    member_role::entities::{AssignMemberRole, MemberRole, UnassignMemberRole},
    role::entities::RoleId,
    server_member::MemberId,
};
pub trait MemberRoleRepository: Send + Sync {
    fn assign(
        &self,
        member_role: AssignMemberRole,
    ) -> impl Future<Output = Result<MemberRole, CoreError>>;
    fn unassign(
        &self,
        member_role: UnassignMemberRole,
    ) -> impl Future<Output = Result<(), CoreError>>;
}

pub trait MemberRoleService: Send + Sync {
    fn assign_member_to_role(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<MemberRole, CoreError>>;
    fn unassign_member_from_role(
        &self,
        role_id: RoleId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<(), CoreError>>;
}

/// Mock implementation of MemberRoleRepository for testing
#[derive(Clone)]
pub struct MockMemberRoleRepository {
    member_role: Arc<Mutex<Vec<MemberRole>>>,
}

impl MockMemberRoleRepository {
    pub fn new() -> Self {
        Self {
            member_role: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockMemberRoleRepository {
    fn default() -> Self {
        Self::new()
    }
}
impl MemberRoleRepository for MockMemberRoleRepository {
    async fn assign(&self, member_role: AssignMemberRole) -> Result<MemberRole, CoreError> {
        let mut member_roles = self.member_role.lock().unwrap();
        let member_role = MemberRole {
            member_id: member_role.member_id,
            role_id: member_role.role_id,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };
        member_roles.push(member_role.clone());
        Ok(member_role)
    }
    async fn unassign(&self, member_role: UnassignMemberRole) -> Result<(), CoreError> {
        let mut member_roles = self.member_role.lock().unwrap();
        member_roles.retain(|mr| {
            !(mr.member_id == member_role.member_id && mr.role_id == member_role.role_id)
        });
        Ok(())
    }
}
