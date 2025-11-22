use std::env;

use axum::middleware::from_fn;
use axum::{Router, routing::get};
use core::create_service;
use sqlx::postgres::PgPoolOptions;

mod http;

use crate::http::friend::routes::friend_routes;
use crate::http::health::health_check;
use crate::http::server::middleware::auth_middleware;
use crate::http::server::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get database URL from environment variable
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/communities".to_string());

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✓ Connected to database");

    // Create service with all dependencies
    let service = create_service(pool);

    // Create application state (shared between both servers)
    let app_state = AppState::new(service);

    // Health server - runs on separate port for DDOS protection
    let health_app = Router::new()
        .route("/health", get(health_check))
        .with_state(app_state.clone());

    // Main API server - for business logic endpoints
    let api_app = Router::<AppState>::new()
        .merge(friend_routes())
        // Future API routes will be added here
        .layer(from_fn(auth_middleware))
        .with_state(app_state.clone());

    // Get ports from environment
    let health_port = env::var("HEALTH_PORT").unwrap_or_else(|_| "9090".to_string());
    let api_port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());

    let health_addr = format!("0.0.0.0:{}", health_port);
    let api_addr = format!("0.0.0.0:{}", api_port);

    println!("🏥 Health server starting on {}", health_addr);
    println!("🚀 API server starting on {}", api_addr);

    // Create TCP listeners for both servers
    let health_listener = tokio::net::TcpListener::bind(&health_addr).await?;
    let api_listener = tokio::net::TcpListener::bind(&api_addr).await?;

    // Run both servers concurrently
    tokio::try_join!(
        axum::serve(health_listener, health_app),
        axum::serve(api_listener, api_app)
    )?;

    Ok(())
}
