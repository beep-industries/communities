use axum::middleware::from_extractor_with_state;
use communities_core::create_repositories;
use sqlx::postgres::PgConnectOptions;

use crate::friend_routes;
use crate::http::server::middleware::auth::AuthMiddleware;
use crate::{
    Config,
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
        let state: AppState = create_repositories(
            PgConnectOptions::new()
                .host(&config.database.host)
                .port(config.database.port)
                .username(&config.database.user)
                .password(&config.database.password)
                .database(&config.database.db_name),
        )
        .await
        .inspect_err(|e| println!("{:?}", e))?
        .into();

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

pub trait AppBuilder {
    fn build(config: Config) -> impl Future<Output = Result<App, ApiError>>;
    fn with_state(self, state: AppState) -> impl Future<Output = Result<App, ApiError>>;
}

impl AppBuilder for App {
    async fn build(config: Config) -> Result<App, ApiError> {
        App::new(config).await
    }

    async fn with_state(mut self, state: AppState) -> Result<App, ApiError> {
        self.state = state;
        Ok(self)
    }
}
