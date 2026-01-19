use beep_authz::SpiceDbObject;

use crate::domain::{common::CoreError, friend::entities::UserId, server::entities::ServerId};

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

    fn can_manage_channels_in_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>>;

    fn can_view_channels_in_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> impl Future<Output = Result<bool, CoreError>>;

    fn can_manage_server(
        &self,
        user_id: UserId,
        server_id: ServerId,
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
}
