use api::app::App;
use api::http::server::ApiError;
use dotenv::dotenv;

use api::config::Config;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut config: Config = Config::parse();
    config.load_routing().map_err(|e| ApiError::StartupError {
        msg: format!("Failed to load routing config: {}", e),
    })?;
    let app = App::new(config).await?;
    app.start().await?;
    Ok(())
}
