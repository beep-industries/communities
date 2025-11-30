use axum::extract::State;
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;

use communities_core::domain::health::port::HealthService;

use crate::http::server::{ApiError, AppState, Response};

/// Response structure for the health check
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database_status: String,
    pub timestamp: String,
}

/// Handler for /health endpoint
/// Checks database connectivity and service health
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Response<HealthResponse>, ApiError> {
    let health_check = state.service.check_health().await?;

    let is_healthy = health_check.value();
    let status = if is_healthy { "healthy" } else { "unhealthy" };
    let database_status = if is_healthy {
        "connected"
    } else {
        "disconnected"
    };

    let response = HealthResponse {
        status: status.to_string(),
        database_status: database_status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(Response::ok(response))
}
