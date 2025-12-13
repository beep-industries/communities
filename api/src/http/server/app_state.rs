use std::sync::Arc;

use beep_auth::KeycloakAuthRepository;
use communities_core::{CommunitiesService, application::CommunitiesState};

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub service: CommunitiesService,
    pub auth_repository: Arc<KeycloakAuthRepository>,
}

impl AppState {
    /// Create a new AppState with the given service
    pub fn new(service: CommunitiesService, auth_repository: Arc<KeycloakAuthRepository>) -> Self {
        Self { service, auth_repository }
    }

    /// Shutdown the underlying database pool
    pub async fn shutdown(&self) {
        self.service.shutdown_pool().await
    }
}

impl From<CommunitiesState> for AppState {
    fn from(repositories: CommunitiesState) -> Self {
        let service = CommunitiesService::new(
            repositories.server_repository.clone(),
            repositories.friendship_repository.clone(),
            repositories.health_repository.clone(),
            repositories.member_repository.clone(),
        );
        let auth_repository = repositories.auth_repository.clone();
        AppState { service, auth_repository }
    }
}
