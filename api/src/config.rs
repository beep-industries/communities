use clap::Parser;
use clap::ValueEnum;
use communities_core::application::MessageRoutingInfos;
use sqlx::postgres::PgConnectOptions;
use std::path::PathBuf;

#[derive(Clone, Parser, Debug, Default)]
#[command(name = "communities-api")]
#[command(about = "Communities API Server", long_about = None)]
pub struct Config {
    #[command(flatten)]
    pub database: DatabaseConfig,

    #[command(flatten)]
    pub jwt: JwtConfig,

    #[command(flatten)]
    pub server: ServerConfig,

    #[arg(
        long = "routing-config",
        env = "ROUTING_CONFIG_PATH",
        default_value = "config/routing.yaml"
    )]
    pub routing_config_path: PathBuf,

    #[arg(skip)]
    pub routing: MessageRoutingInfos,

    #[arg(
        long = "environment",
        env = "ENVIRONMENT",
        default_value = "development"
    )]
    pub environment: Environment,
}

impl Config {
    /// Load routing configuration from YAML file
    pub fn load_routing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = std::fs::read_to_string(&self.routing_config_path)?;
        self.routing = serde_yaml::from_str(&yaml_content)?;
        Ok(())
    }
}
#[derive(Clone, Parser, Debug, Default)]
pub struct DatabaseConfig {
    #[arg(
        long = "database-host",
        env = "DATABASE_HOST",
        default_value = "localhost"
    )]
    pub host: String,

    #[arg(long = "database-port", env = "DATABASE_PORT", default_value = "5432")]
    pub port: u16,

    #[arg(
        long = "database-user",
        env = "DATABASE_USER",
        default_value = "postgres"
    )]
    pub user: String,

    #[arg(
        long = "database-password",
        env = "DATABASE_PASSWORD",
        value_name = "database_password"
    )]
    pub password: String,

    #[arg(
        long = "database-name",
        env = "DATABASE_NAME",
        default_value = "communities",
        value_name = "database_name"
    )]
    pub db_name: String,
}

impl Into<PgConnectOptions> for DatabaseConfig {
    fn into(self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .password(&self.password)
            .database(&self.db_name)
    }
}
#[derive(Clone, Parser, Debug, Default)]
pub struct JwtConfig {
    #[arg(
        long = "jwt-secret-key",
        env = "JWT_SECRET_KEY",
        name = "jwt_secret_key"
    )]
    pub secret_key: String,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct ServerConfig {
    #[arg(
        long = "server-api-port",
        env = "API_PORT",
        default_value = "8080",
        name = "api_port"
    )]
    pub api_port: u16,

    #[arg(
        long = "server-health-port",
        env = "HEALTH_PORT",
        default_value = "8081"
    )]
    pub health_port: u16,
}

#[derive(Clone, Debug, ValueEnum, Default)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Test
}
