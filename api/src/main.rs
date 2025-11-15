use axum::Router;
use core::{
    PostgresFriendshipRepository, PostgresHealthRepository, PostgresServerRepository, Service,
};
use sqlx::postgres::PgPoolOptions;
use std::env;

mod http;

use http::{health::health_routes, server::AppState};

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

    // Create repository instances with dependency injection
    let server_repository = PostgresServerRepository::new(pool.clone());
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool.clone());

    // Create service with dependency injection
    let service = Service::new(server_repository, friendship_repository, health_repository);

    // Create application state
    let app_state = AppState::new(service);

    // Build router with health routes
    let app = Router::new().merge(health_routes()).with_state(app_state);

    // Get port from environment or use default
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Server starting on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
