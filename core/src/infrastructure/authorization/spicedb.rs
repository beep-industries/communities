use beep_authz::SpiceDbRepository;

use crate::domain::{authorization::ports::AuthorizationRepository, common::CoreError};

#[derive(Debug, Clone)]
pub struct SpiceDbAuthorizationRepository {
    client: SpiceDbRepository,
}

impl SpiceDbAuthorizationRepository {
    pub fn new(client: SpiceDbRepository) -> Self {
        Self { client }
    }
}

impl AuthorizationRepository for SpiceDbAuthorizationRepository {
    async fn check_authz(
        &self,
        user: beep_authz::SpiceDbObject,
        permission: beep_authz::Permissions,
        resource: beep_authz::SpiceDbObject,
    ) -> Result<bool, CoreError> {
        self.client
            .check_permissions(resource, permission, user)
            .await
            .result()
            .map_err(|e| {
                dbg!(e);
                CoreError::Forbidden
            })?;
        Ok(true)
    }
}
