use core::domain::common::services::Service;
use core::domain::{
    friend::ports::FriendshipRepository, health::port::HealthRepository,
    server::ports::ServerRepository,
};
use std::sync::Arc;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    pub service: Arc<Service<S, F, H>>,
}

impl<S, F, H> AppState<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    /// Create a new AppState with the given service
    pub fn new(service: Service<S, F, H>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}
