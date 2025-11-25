pub mod api_error;
pub mod app_state;
pub mod middleware;
pub mod response;

use core::create_service;

pub use api_error::ApiError;
pub use app_state::AppState;
pub use response::Response;
use sqlx::postgres::PgPoolOptions;

use crate::Config;


pub async fn create_app_state(config: Config) -> Result<AppState, ApiError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(config.database.into())
        .await
        .map_err(|_| ApiError::StartupError("Failed to connect to the database".to_string()))?;

    // Create service with all dependencies
    let service = create_service(pool);

    // Create application state (shared between both servers)
    let app_state = AppState::new(service);
    Ok(app_state)
}
