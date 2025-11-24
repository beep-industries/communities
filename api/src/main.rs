use axum::Router;
use axum::middleware::from_fn;
use communities_core::create_service;
use sqlx::postgres::PgPoolOptions;

mod http;

use crate::http::friend::routes::friend_routes;
use crate::http::health::routes::health_routes;
use crate::http::server::AppState;
use crate::http::server::middleware::auth_middleware;

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = Config::parse();
    println!("{:#?}", config);
    // Get database URL from environment variable
    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(config.database.into())
        .await?;

    println!("✓ Connected to database");

    // Create service with all dependencies
    let service = create_service(pool);

    // Create application state (shared between both servers)
    let app_state = AppState::new(service);

    // Health server - runs on separate port for DDOS protection
    let health_app = Router::new()
        .merge(health_routes())
        .with_state(app_state.clone());

    // Main API server - for business logic endpoints
    let api_app = Router::<AppState>::new()
        .merge(friend_routes())
        // Future API routes will be added here
        .layer(from_fn(auth_middleware))
        .with_state(app_state.clone());

    // Get ports from environment

    let health_addr = format!("0.0.0.0:{}", config.server.health_port);
    let api_addr = format!("0.0.0.0:{}", config.server.api_port);

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
