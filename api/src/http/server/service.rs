use core::application::CommunitiesService;

use super::AppState;

/// Concrete AppState type using PostgreSQL repositories
pub type ConcreteAppState = AppState<
    core::infrastructure::server::repositories::postgres::PostgresServerRepository,
    core::infrastructure::friend::repositories::postgres::PostgresFriendshipRepository,
    core::infrastructure::health::repositories::postgres::PostgresHealthRepository,
>;

impl ConcreteAppState {
    /// Create a new ConcreteAppState from CommunitiesService
    pub fn from_service(service: CommunitiesService) -> Self {
        Self::new(service)
    }
}
