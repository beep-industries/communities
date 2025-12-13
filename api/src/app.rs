use communities_core::create_state;
use sqlx::postgres::PgConnectOptions;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};
use beep_auth::KeycloakAuthRepository;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    Config, friend_routes,
    http::{
        health::routes::health_routes,
        server::{
            ApiError, AppState,
            middleware::auth::auth_middleware,
        },
    },
    server_member_routes, server_routes,
};

#[derive(OpenApi)]
#[openapi(info(
    title = "Beep communities openapi",
    contact(name = "communities-core@beep.ovh"),
    description = "API documentation for the Communities service",
    version = "0.0.1"
))]
struct ApiDoc;
pub struct App {
    config: Config,
    pub state: AppState,
    app_router: axum::Router<AppState>,
    health_router: axum::Router,
}

impl App {
    pub async fn new(config: Config) -> Result<Self, ApiError> {
        let auth_repository = KeycloakAuthRepository::new(
    format!(
                "{}/realms/{}",
                config.keycloak.internal_url, config.keycloak.realm
            ),
            None,
        );
        let state: AppState = create_state(
            PgConnectOptions::new()
                .host(&config.database.host)
                .port(config.database.port)
                .username(&config.database.user)
                .password(&config.database.password)
                .database(&config.database.db_name),
            auth_repository,
                config.clone().routing,
        )
        .await
        .map_err(|e| ApiError::StartupError {
            msg: format!("Failed to create repositories: {}", e),
        })?
        .into();

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let (app_router, mut api) = OpenApiRouter::<AppState>::new()
            .merge(friend_routes())
            .merge(server_routes())
            .merge(server_member_routes())
            // Add application routes here
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .layer(cors)
            .split_for_parts();

        // Override API documentation info
        let custom_info = ApiDoc::openapi();
        api.info = custom_info.info;

        // let openapi_json = api.to_pretty_json().map_err(|e| ApiError::StartupError {
        //     msg: format!("Failed to generate OpenAPI spec: {}", e),
        // })?;

        let app_router = app_router
            .merge(Scalar::with_url("/scalar", api));

        let health_router = axum::Router::new()
            .merge(health_routes())
            .with_state(state.clone());
        Ok(Self {
            config,
            state,
            app_router,
            health_router,
        })
    }

    pub fn app_router(&self) -> axum::Router<AppState> {
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
            axum::serve(health_listener, self.health_router.clone().into_make_service()),
            axum::serve(api_listener, self.app_router.clone().with_state(self.state.clone()).into_make_service())
        )
        .expect("Failed to start servers");
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.state.shutdown().await;
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
