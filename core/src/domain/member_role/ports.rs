pub trait MemberRoleRepository: Send + Sync {}

pub trait MemberRoleService: Send + Sync {}

/// Mock implementation of MemberRoleRepository for testing
#[derive(Clone)]
pub struct MockMemberRoleRepository {}

impl MemberRoleRepository for MockMemberRoleRepository {}
impl MockMemberRoleRepository {}
