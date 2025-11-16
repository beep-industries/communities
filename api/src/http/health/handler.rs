use axum::extract::State;
use chrono::Utc;
use serde::Serialize;

use core::domain::health::port::HealthService;

use crate::http::server::{ApiError, ConcreteAppState, Response};

/// Response structure for the readiness health check
#[derive(Debug, Clone, Serialize)]
pub struct ReadinessResponse {
    pub is_healthy: bool,
    pub status: String,
    pub database_status: String,
    pub timestamp: String,
}

/// Response structure for the liveness health check
#[derive(Debug, Clone, Serialize)]
pub struct LivenessResponse {
    pub status: String,
    pub timestamp: String,
    pub message: String,
}

/// Handler for /health/ready endpoint
/// Checks database connectivity and service readiness
pub async fn health_ready(
    State(state): State<ConcreteAppState>,
) -> Result<Response<ReadinessResponse>, ApiError> {
    let health_check = state.service.check_health().await?;

    let is_healthy = health_check.value();
    let status = if is_healthy { "healthy" } else { "unhealthy" };
    let database_status = if is_healthy { "healthy" } else { "unhealthy" };

    let response = ReadinessResponse {
        is_healthy,
        status: status.to_string(),
        database_status: database_status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(Response::ok(response))
}

/// Handler for /health/live endpoint
/// Simple liveness check without database dependency
pub async fn health_live(
    State(_state): State<ConcreteAppState>,
) -> Result<Response<LivenessResponse>, ApiError> {
    let response = LivenessResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        message: "Service is live".to_string(),
    };

    Ok(Response::ok(response))
}
