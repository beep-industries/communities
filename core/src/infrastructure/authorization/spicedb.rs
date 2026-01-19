use beep_authz::SpiceDbRepository;

use crate::domain::authorization::ports::AuthorizationRepository;

#[derive(Clone)]
pub struct SpiceDbAuthorizationRepository {
    client: SpiceDbRepository,
}

impl SpiceDbAuthorizationRepository {
    pub fn new(client: SpiceDbRepository) -> Self {
        Self { client }
    }
}

impl AuthorizationRepository for SpiceDbAuthorizationRepository {
    // Methods will be implemented later
}
