use axum::{Router, routing::get};


use crate::http::server::AppState;

use super::handler::{health_live, health_ready};

/// Creates and configures health check routes
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
}
