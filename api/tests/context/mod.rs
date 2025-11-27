use api::{
    App, Config,
    app::AppBuilder,
    config::{DatabaseConfig, JwtConfig},
};
use axum_test::TestServer;
use communities_core::{application::CommunitiesRepositories, create_repositories};
use test_context::AsyncTestContext;

pub struct TestContext {
    pub app: App,
    pub test_router: TestServer,
    pub repositories: CommunitiesRepositories,
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

        let server = api::config::ServerConfig {
            api_port: 8080,
            health_port: 8081,
        };

        let config = Config {
            database,
            jwt,
            server,
        };

        let repositories = create_repositories(config.clone().database.into())
            .await
            .expect("Failed to create repositories");



        let app = App::build(config)
            .await
            .inspect_err(|e| eprintln!("Error building app: {}", e))
            .expect("Failed to build app")
            .with_state(repositories.clone().into())
            .await
            .expect("Failed to set state");

        let test_router = TestServer::new(app.app_router()).unwrap();

        TestContext {
            app,
            test_router,
            repositories,
        }
    }

    async fn teardown(self) {
        // Teardown code after each test
    }
}
