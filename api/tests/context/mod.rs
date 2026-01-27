use api::config::{
    BeepServicesConfig as BeepServicesConfigApi, ContentConfiguration, Environment, KeycloakConfig,
    SpiceConfig,
};
use api::{App, Config, app::AppBuilder, config::DatabaseConfig};
use axum_test::TestServer;
use base64::{Engine as _, engine::general_purpose};
use communities_core::application::{BeepServicesConfig, MessageRoutingConfig};
use communities_core::{application::CommunitiesRepositories, create_repositories_with_mock_authz};
use outbox_dispatch::lapin::RabbitClientConfig;
use serde_json::Value;
use test_context::AsyncTestContext;
use uuid::Uuid;

use super::helpers::auth::{generate_mock_token, get_keycloak_token};

pub struct TestContext {
    pub app: App,
    // Router without auth token (will get 401 unauthorized)
    pub unauthenticated_router: TestServer,
    // Router with auth token (authenticated as test user)
    pub authenticated_router: TestServer,
    pub repositories: CommunitiesRepositories,
    pub authenticated_user_id: Uuid,
    pub test_token: String,
}

impl TestContext {
    /// Extract user ID from JWT token
    fn extract_user_id_from_token(token: &str) -> Option<Uuid> {
        // JWT structure: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        // Decode the payload (second part)
        let payload = general_purpose::STANDARD_NO_PAD.decode(parts[1]).ok()?;

        let payload_str = String::from_utf8(payload).ok()?;
        let payload_json: Value = serde_json::from_str(&payload_str).ok()?;

        // Extract 'sub' claim which contains the user ID
        let sub = payload_json.get("sub")?.as_str()?;
        Uuid::parse_str(sub).ok()
    }

    /// Try to get a real Keycloak token, fallback to mock token if Keycloak is not available
    /// Returns (token, user_id)
    async fn get_or_generate_test_token(
        keycloak_url: &str,
        realm: &str,
        fallback_user_id: &Uuid,
    ) -> (String, Uuid) {
        // Try to get a real token from Keycloak
        // Note: This requires a test user to exist in Keycloak
        match get_keycloak_token(
            keycloak_url,
            realm,
            "user-service",
            "ABvykyIUah2CcQPiRcvcgd7GA4MrEdx4",
            "testuser",
            "testpassword",
        )
        .await
        {
            Ok(token) => {
                println!("Using real Keycloak token for tests");
                // Extract user ID from the token
                let user_id = Self::extract_user_id_from_token(&token).unwrap_or(*fallback_user_id);
                (token, user_id)
            }
            Err(e) => {
                println!(
                    "Warning: Could not get Keycloak token ({}), using mock token. \
                     Some tests may fail. Start Keycloak with docker-compose for full integration tests.",
                    e
                );
                let token = generate_mock_token(fallback_user_id);
                (token, *fallback_user_id)
            }
        }
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

        let server = api::config::ServerConfig {
            api_port: 8080,
            health_port: 8081,
        };

        let cors_origins = vec!["http://localhost:3003".to_string()];

        let beep_services = BeepServicesConfigApi {
            user_service_url: "http://localhost:3000".to_string(),
        };

        let keycloak_url = "http://localhost:8080";
        let keycloak_realm = "myrealm";
        let rabbit = RabbitClientConfig {
            uri: "amqp://localhost:5672".to_string(),
        };
        let mut config = Config {
            rabbit,
            database,
            server,
            origins: cors_origins,
            routing_config_path: "../config/routing.yaml".to_string().into(),
            routing: MessageRoutingConfig::default(),
            beep_services,
            environment: Environment::Test,
            keycloak: KeycloakConfig {
                internal_url: keycloak_url.to_string(),
                realm: keycloak_realm.to_string(),
            },
            spicedb: SpiceConfig {
                endpoint: "http://localhost:50051".to_string(),
                token: "foobar".to_string(),
            },
            content_config: ContentConfiguration {
                url: "https://localhost:1234".to_string(),
            },
        };

        config
            .load_routing()
            .expect("Could not load the routing config");

        // Use the test repository creation function with mock authorization
        let repositories = create_repositories_with_mock_authz(
            config.clone().database.into(),
            config.clone().routing,
            format!(
                "{}/realms/{}",
                config.keycloak.internal_url, config.keycloak.realm
            ),
            BeepServicesConfig {
                user_service_url: config.beep_services.user_service_url.clone(),
            },
            config.clone().spicedb.into(),
        )
        .await
        .expect("Failed to create repositories");

        let app = App::build(config.clone())
            .await
            .expect("Failed to build app")
            .with_state(repositories.clone().into())
            .await
            .expect("Failed to set state");

        // Generate fallback user ID (will be overridden by actual Keycloak user ID if available)
        let fallback_user_id = Uuid::new_v4();

        // Get token from Keycloak or generate mock token
        let (test_token, authenticated_user_id) =
            Self::get_or_generate_test_token(keycloak_url, keycloak_realm, &fallback_user_id).await;

        // Build unauthenticated router (without auth token)
        let unauthenticated_router = TestServer::new(app.app_router()).unwrap();

        // Build authenticated router (with auth token)
        let mut authenticated_router = TestServer::new(app.app_router()).unwrap();
        authenticated_router.add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", test_token),
        );

        TestContext {
            app,
            unauthenticated_router,
            authenticated_router,
            repositories,
            authenticated_user_id,
            test_token,
        }
    }

    async fn teardown(self) {
        self.app.shutdown().await;
    }
}

impl TestContext {
    /// Create a new authenticated router with a different user ID
    /// This is useful for testing access control between different users
    /// Note: When using real Keycloak, you'd need to create additional test users
    pub async fn create_authenticated_router_with_different_user(&self) -> TestServer {
        let (router, _) = self
            .create_authenticated_router_with_different_user_and_id()
            .await;
        router
    }

    /// Create an authenticated router for a different user, returning both the router and user ID
    pub async fn create_authenticated_router_with_different_user_and_id(
        &self,
    ) -> (TestServer, Uuid) {
        let fallback_user_id = Uuid::new_v4();

        // Try to get token for a different user, or use mock token
        let different_token = match get_keycloak_token(
            "http://localhost:8080",
            "myrealm",
            "user-service",
            "ABvykyIUah2CcQPiRcvcgd7GA4MrEdx4",
            "testuser2", // Different test user
            "testpassword",
        )
        .await
        {
            Ok(token) => token,
            Err(_) => generate_mock_token(&fallback_user_id),
        };

        // Extract user ID from token
        let user_id =
            Self::extract_user_id_from_token(&different_token).unwrap_or(fallback_user_id);

        let mut router = TestServer::new(self.app.app_router()).unwrap();
        router.add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", different_token),
        );
        (router, user_id)
    }

    /// Get the current authenticated user's ID
    pub fn authenticated_user_id(&self) -> Uuid {
        self.authenticated_user_id
    }

    /// Get the current test token
    pub fn test_token(&self) -> &str {
        &self.test_token
    }
}
