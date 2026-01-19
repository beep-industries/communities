pub trait AuthorizationRepository: Send + Sync {
    // Methods will be added later
}

#[derive(Clone)]
pub struct MockAuthorizationRepository;

impl MockAuthorizationRepository {
    pub fn new() -> Self {
        Self
    }
}

impl AuthorizationRepository for MockAuthorizationRepository {
    // Methods will be implemented later
}
