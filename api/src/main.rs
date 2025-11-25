use axum::Router;
use axum::middleware::from_extractor_with_state;
use axum::middleware::from_fn;
use communities_core::create_service;
use core::create_service;
use dotenv::dotenv;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

mod http;

use crate::http::friend::routes::friend_routes;
use crate::http::health::routes::health_routes;
use crate::http::server::middleware::auth::AuthMiddleware;
use crate::http::server::middleware::auth::entities::AuthValidator;
use crate::http::server::{ApiError, AppState, create_app_state};

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Load environment variables from .env file
    dotenv().ok();

    let config: Config = Config::parse();
    let app_state: AppState = create_app_state(config.clone()).await?;
    // Health server - runs on separate port for DDOS protection
    let health_app = Router::new()
        .merge(health_routes())
        .with_state(app_state.clone());

    let token_validator = AuthValidator::new(config.clone().jwt.secret_key);
    // Main API server - for business logic endpoints
    let api_app = Router::<AppState>::new()
        .merge(friend_routes())
        // Future API routes will be added here
        .route_layer(from_extractor_with_state::<AuthMiddleware, AuthValidator>(
            token_validator,
        ))
        .with_state(app_state.clone());

    // Get ports from environment

    let health_addr = format!("0.0.0.0:{}", config.clone().server.health_port);
    let api_addr = format!("0.0.0.0:{}", config.clone().server.api_port);

    println!("🏥 Health server starting on {}", health_addr);
    println!("🚀 API server starting on {}", api_addr);

    // Create TCP listeners for both servers
    let health_listener = tokio::net::TcpListener::bind(&health_addr)
        .await
        .map_err(|_| ApiError::StartupError {
            msg: format!("Failed to bind health server: {}", health_addr),
        })?;
    let api_listener = tokio::net::TcpListener::bind(&api_addr)
        .await
        .map_err(|_| ApiError::StartupError {
            msg: format!("Failed to bind API server: {}", api_addr),
        })?;

    // Run both servers concurrently
    tokio::try_join!(
        axum::serve(health_listener, health_app),
        axum::serve(api_listener, api_app)
    )
    .expect("Failed to start servers");

    Ok(())
}
