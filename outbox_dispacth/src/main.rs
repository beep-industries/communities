use communities_core::{
    application::MessageRoutingConfig, domain::{outbox::ports::OutboxRepository, server::entities::InsertServerInput}, infrastructure::outbox::postgres::PostgresOutboxRepository
};
use futures_util::StreamExt;
use outbox_dispacth::dispatch::Dispatcher;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    // Database connection string from .env file
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    println!("Connecting to database: {}", database_url);

    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database successfully!");

    // Instantiate the PostgreSQL outbox repository
    let outbox_repo = PostgresOutboxRepository::new(pool);

    println!("Starting to listen for outbox notifications...");
    println!("Waiting for messages on 'outbox_channel'...\n");

    // Listen to the outbox stream
    let mut stream = outbox_repo.listen_outbox_event().await?;
    let dispatcher = Dispatcher::new(stream,MessageRoutingConfig::default(),)
    // Display strings from the stream
    while let Some(result) = stream.next().await {
        let notif = match result {
            Ok(notification_payload) => notification_payload,
            Err(e) => {
                eprintln!("❌ Error receiving notification: {:?}", e);
                continue;
            }
        };

        match notif.payload::<InsertServerInput>() {
            Ok(value) => println!("{:?}", value),
            Err(e) => eprintln!("{:?}", e),
        }
    }

    println!("Stream ended.");

    Ok(())
}
