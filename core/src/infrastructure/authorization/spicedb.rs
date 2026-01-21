use beep_authz::SpiceDbRepository;

use crate::domain::{authorization::ports::{AuthorizationRepository, MockAuthorizationRepository}, common::CoreError};

#[derive(Debug, Clone)]
pub enum SpiceDbAuthorizationRepository {
    Real(SpiceDbRepository),
    Mock(MockAuthorizationRepository),
}

impl SpiceDbAuthorizationRepository {
    pub fn new(client: SpiceDbRepository) -> Self {
        Self::Real(client)
    }
    
    pub fn new_mock() -> Self {
        Self::Mock(MockAuthorizationRepository::new())
    }
}

impl AuthorizationRepository for SpiceDbAuthorizationRepository {
    async fn check_authz(
        &self,
        user: beep_authz::SpiceDbObject,
        permission: beep_authz::Permissions,
        resource: beep_authz::SpiceDbObject,
    ) -> Result<bool, CoreError> {
        match self {
            Self::Real(client) => {
                client
                    .check_permissions(resource, permission, user)
                    .await
                    .result()
                    .map_err(|_| CoreError::Forbidden)?;
                Ok(true)
            }
            Self::Mock(mock) => mock.check_authz(user, permission, resource).await,
        }
    }
}
