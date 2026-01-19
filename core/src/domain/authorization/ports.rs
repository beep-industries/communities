use beep_authz::SpiceDbObject;

use crate::domain::{common::CoreError, friend::entities::UserId};

pub trait AuthorizationRepository: Send + Sync {
    fn check_authz(
        &self,
        user: SpiceDbObject,
        permission: beep_authz::Permissions,
        resource: SpiceDbObject,
    ) -> impl Future<Output = Result<bool, CoreError>>;
}

pub trait AuthorizationService: Send + Sync {
    fn check_authz(
        &self,
        user_id: UserId,
        permission: beep_authz::Permissions,
        resource: SpiceDbObject,
    ) -> impl Future<Output = Result<bool, CoreError>>;
}

#[derive(Clone)]
pub struct MockAuthorizationRepository;

impl MockAuthorizationRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockAuthorizationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthorizationRepository for MockAuthorizationRepository {
    async fn check_authz(
        &self,
        _user_id: SpiceDbObject,
        _permission: beep_authz::Permissions,
        _object: SpiceDbObject,
    ) -> Result<bool, CoreError> {
        Ok(true)
    }
    // Methods will be implemented later
}
