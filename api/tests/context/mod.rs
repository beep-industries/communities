use api::config::Environment;
use api::http::server::middleware::auth::entities::Claims;
use api::{
    App, Config,
    app::AppBuilder,
    config::{DatabaseConfig, JwtConfig},
};
use axum_extra::extract::cookie::Cookie;
use axum_test::TestServer;
use chrono::Utc;
use communities_core::application::MessageRoutingInfos;
use communities_core::{application::CommunitiesRepositories, create_repositories};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use test_context::AsyncTestContext;
use uuid::Uuid;

pub struct TestContext {
    pub app: App,
    // Router protected by auth middleware (requires access_token cookie)
    pub unauthenticated_router: TestServer,
    // Router without auth middleware for unauthenticated tests
    pub authenticated_router: TestServer,
    pub repositories: CommunitiesRepositories,
    pub jwt: JwtMaker,
    pub authenticated_user_id: Uuid,
}

#[derive(Clone)]
pub struct JwtMaker {
    secret: String,
}

impl JwtMaker {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Create an HS256 JWT suitable for the auth middleware cookie
    pub fn make_for_user(&self, user_id: Uuid, ttl_secs: i64) -> String {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + ttl_secs,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .expect("Failed to encode JWT for tests")
    }
}

impl AsyncTestContext for TestContext {
    async fn setup() -> Self {
        let database: DatabaseConfig = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password: "password".to_string(),
            db_name: "communities".to_string(),
        };

        let jwt = JwtConfig {
            secret_key: "test_secret_key".to_string(),
        };
        let test_secret = jwt.secret_key.clone();

        let server = api::config::ServerConfig {
            api_port: 8080,
            health_port: 8081,
        };

        let cors_origins = vec!["http://localhost:3000".to_string()];

        let config = Config {
            database,
            jwt,
            server,
            origins: cors_origins,
            routing_config_path: "tests/config/routing_config.yaml".to_string().into(),
            routing: MessageRoutingInfos::default(),
            environment: Environment::Test,
        };

        let repositories =
            create_repositories(config.clone().database.into(), config.clone().routing)
                .await
                .expect("Failed to create repositories");

        let app = App::build(config)
            .await
            .expect("Failed to build app")
            .with_state(repositories.clone().into())
            .await
            .expect("Failed to set state");

        let jwt = JwtMaker::new(test_secret);
        let authenticated_user_id = Uuid::new_v4();
        let token = jwt.make_for_user(authenticated_user_id, 3600);
        let cookie = Cookie::new("access_token", token);
        // Build authenticated router (with middleware)
        let unauthenticated_router = TestServer::new(app.app_router()).unwrap();

        // Build unauthenticated router (no auth middleware)
        let mut authenticated_router = TestServer::new(app.app_router()).unwrap();

        authenticated_router.add_cookie(cookie);
        TestContext {
            app,
            unauthenticated_router,
            authenticated_router,
            repositories,
            jwt,
            authenticated_user_id,
        }
    }

    async fn teardown(self) {
        self.app.shutdown().await;
    }
}

impl TestContext {
    /// Create a new authenticated router with a different user ID
    pub async fn create_authenticated_router_with_different_user(&self) -> TestServer {
        let token = self.jwt.make_for_user(Uuid::new_v4(), 3600);
        let cookie = Cookie::new("access_token", token);
        let mut router = TestServer::new(self.app.app_router()).unwrap();
        router.add_cookie(cookie);
        router
    }
}
