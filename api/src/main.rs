use api::app::App;
use api::http::server::ApiError;
use dotenv::dotenv;

mod http;

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Load environment variables from .env file
    dotenv().ok();

    let config: Config = Config::parse();
    let app = App::new(config).await?;
    app.start().await?;
    Ok(())
}
