use std::ops::Deref;

use serde::Serialize;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{domain::common::CoreError, write_outbox_event};

/// A record representing an outbox event to be published to a message broker.
///
/// This struct encapsulates an event payload along with routing information,
/// following the transactional outbox pattern to ensure reliable message delivery.
///
/// # Type Parameters
///
/// * `TPayload` - The type of the event payload, must be serializable
/// * `TRouter` - The type that provides message routing information
pub struct OutboxEventRecord<TPayload: Serialize, TRouter: MessageRouter> {
    /// Unique identifier for this outbox event
    pub id: Uuid,
    /// Router providing exchange and routing key information
    pub router: TRouter,
    /// The event payload to be serialized and published
    pub payload: TPayload,
}

impl<TPayload: Serialize + Clone, TRouter: MessageRouter> OutboxEventRecord<TPayload, TRouter> {
    /// Creates a new outbox event record with a generated UUID.
    ///
    /// # Arguments
    ///
    /// * `router` - The message router providing routing information
    /// * `payload` - The event payload to be published
    ///
    /// # Returns
    ///
    /// A new `OutboxEventRecord` with a randomly generated UUID
    pub fn new(router: TRouter, payload: TPayload) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            id: uuid,
            router,
            payload,
        }
    }

    /// Writes this outbox event to the database.
    ///
    /// # Arguments
    ///
    /// * `executor` - A PostgreSQL executor (connection or transaction)
    ///
    /// # Returns
    ///
    /// The UUID of the written event on success, or a `CoreError` on failure
    ///
    /// # Errors
    ///
    /// Returns an error if the database write operation fails
    pub async fn write(&self, executor: impl PgExecutor<'_>) -> Result<Uuid, CoreError> {
        write_outbox_event(executor, self).await
    }
}

/// Message routing information containing the exchange name and routing key.
///
/// This struct encapsulates the routing metadata required to publish
/// a message to the correct destination in a message broker.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MessageRoutingInfo(ExchangeName);

impl Deref for MessageRoutingInfo {
    type Target = ExchangeName;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MessageRoutingInfo {
    /// Creates a new `MessageRoutingInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `exchange_name` - The name of the message broker exchange
    /// * `routing_key` - The routing key for message delivery
    pub fn new(exchange_name: &str) -> Self {
        Self(exchange_name.to_string())
    }
}

/// Trait for types that can provide message routing information.
///
/// Implementors must provide both an exchange name and routing key
/// for publishing messages to a message broker.
pub trait MessageRouter {
    fn exchange_name(&self) -> String;
}

impl MessageRouter for MessageRoutingInfo {
    fn exchange_name(&self) -> String {
        self.0.clone()
    }
}
impl<TPayload: Serialize, TRouter: MessageRouter> Serialize
    for OutboxEventRecord<TPayload, TRouter>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.payload.serialize(serializer)
    }
}

pub type ExchangeName = String;
