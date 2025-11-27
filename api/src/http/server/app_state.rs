use communities_core::{CommunitiesService, application::CommunitiesRepositories};

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub service: CommunitiesService,
}

impl AppState {
    /// Create a new AppState with the given service
    pub fn new(service: CommunitiesService) -> Self {
        Self { service }
    }
}

impl From<CommunitiesRepositories> for AppState {
    fn from(repositories: CommunitiesRepositories) -> Self {
        let service = CommunitiesService::new(
            repositories.server_repository,
            repositories.friendship_repository,
            repositories.health_repository,
        );
        AppState { service }
    }
}
