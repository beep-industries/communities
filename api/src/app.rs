use axum::middleware::from_extractor_with_state;

use crate::http::friend::routes::friend_routes;
use crate::http::server::middleware::auth::AuthMiddleware;
use crate::{
    Config, create_app_state,
    http::{
        health::routes::health_routes,
        server::{ApiError, AppState, middleware::auth::entities::AuthValidator},
    },
};

pub struct App {
    config: Config,
    pub state: AppState,
    pub auth_validator: AuthValidator,
    app_router: axum::Router,
    health_router: axum::Router,
}

impl App {
    pub async fn new(config: Config) -> Result<Self, ApiError> {
        let state = create_app_state(config.clone()).await?;
        let auth_validator = AuthValidator::new(config.clone().jwt.secret_key);
        let app_router = axum::Router::<AppState>::new()
            .merge(friend_routes())
            // Add application routes here
            .route_layer(from_extractor_with_state::<AuthMiddleware, AuthValidator>(
                auth_validator.clone(),
            ))
            .with_state(state.clone());
        let health_router = axum::Router::new()
            .merge(health_routes())
            .with_state(state.clone());
        Ok(Self {
            config,
            state,
            auth_validator,
            app_router,
            health_router,
        })
    }

    pub fn app_router(&self) -> axum::Router {
        self.app_router.clone()
    }

    pub async fn start(&self) -> Result<(), ApiError> {
        let health_addr = format!("0.0.0.0:{}", self.config.clone().server.health_port);
        let api_addr = format!("0.0.0.0:{}", self.config.clone().server.api_port);
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
            axum::serve(health_listener, self.health_router.clone()),
            axum::serve(api_listener, self.app_router.clone())
        )
        .expect("Failed to start servers");
        Ok(())
    }
}
