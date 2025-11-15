use axum::{routing::get, Router};

use super::handler::{health_live, health_ready};
use crate::http::server::AppState;

/// Creates and configures health check routes
pub fn health_routes<S, F, H>() -> Router<AppState<S, F, H>>
where
    S: core::domain::server::ports::ServerRepository + Clone + Send + Sync + 'static,
    F: core::domain::friend::ports::FriendshipRepository + Clone + Send + Sync + 'static,
    H: core::domain::health::port::HealthRepository + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
}
