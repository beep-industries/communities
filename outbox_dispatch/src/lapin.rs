use clap::Parser;
use lapin::{
    Channel, Connection, options::BasicPublishOptions, publisher_confirm::PublisherConfirm,
};
use thiserror::Error;
use tracing::{debug, error, info, instrument};

pub struct RabbitClient {
    connection: Connection,
    channel: Channel,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct RabbitClientConfig {
    #[arg(
        long = "rabbit-uri",
        env = "RABBIT_URI",
        default_value = "amqp://localhost:5672"
    )]
    pub uri: String,
}

pub type ExchangeName = String;

#[derive(Debug, Error)]
pub enum RabbitClientError {
    #[error("Service could not start: {msg}")]
    StartupError { msg: String },

    #[error("Could not publish message: {msg}")]
    PublishError { msg: String },
}

impl RabbitClient {
    #[instrument(skip_all, fields(uri = %config.uri))]
    pub async fn new(config: RabbitClientConfig) -> Result<Self, RabbitClientError> {
        info!("Connecting to RabbitMQ");
        let connection = Connection::connect(&config.uri, lapin::ConnectionProperties::default())
            .await
            .map_err(|e| {
                error!("Failed to connect to RabbitMQ: {}", e);
                RabbitClientError::StartupError { msg: e.to_string() }
            })?;
        info!("RabbitMQ connection established");

        debug!("Creating RabbitMQ channel");
        let channel = connection.create_channel().await.map_err(|e| {
            error!("Failed to create RabbitMQ channel: {}", e);
            RabbitClientError::StartupError { msg: e.to_string() }
        })?;
        info!("RabbitMQ channel created successfully");

        Ok(RabbitClient {
            connection,
            channel,
        })
    }

    #[instrument(skip_all)]
    pub async fn shutdown(&self) -> Result<(), RabbitClientError> {
        info!("Shutting down RabbitMQ connection");
        self.connection.close(0, "Shutdown").await.map_err(|e| {
            error!("Failed to shutdown RabbitMQ connection: {}", e);
            RabbitClientError::StartupError { msg: e.to_string() }
        })?;
        info!("RabbitMQ connection closed");
        Ok(())
    }

    pub async fn produce(
        &self,
        exchange: &ExchangeName,
        message: &[u8],
    ) -> Result<(), RabbitClientError> {
        let _: PublisherConfirm = self
            .channel
            .basic_publish(
                exchange,
                "",
                BasicPublishOptions::default(),
                message,
                lapin::BasicProperties::default(),
            )
            .await
            .map_err(|e| RabbitClientError::PublishError { msg: e.to_string() })?;
        Ok(())
    }
}
