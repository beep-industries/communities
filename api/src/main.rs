use api::app::App;
use api::http::server::ApiError;
use dotenv::dotenv;

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Load environment variables from .env file
    dotenv().ok();

    let mut config: Config = Config::parse();
    config.load_routing()
        .map_err(|e| ApiError::StartupError {
            msg: format!("Failed to load routing config: {}", e),
        })?;
    let app = App::new(config).await?;
    app.start().await?;
    Ok(())
}
