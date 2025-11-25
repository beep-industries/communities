use serde::Serialize;

/// Trait for domain events that can be written to the outbox.
///
/// Implement this trait on your event types to enable transactional outbox writes.
/// The event type must also implement `Serialize` so it can be stored as JSONB.
///
/// # Example
///
/// ```rust
/// use serde::Serialize;
/// use communities_core::infrastructure::outbox::OutboxEvent;
///
/// #[derive(Serialize)]
/// struct FriendCreatedEvent {
///     friend_id: uuid::Uuid,
///     created_at: chrono::DateTime<chrono::Utc>,
/// }
///
/// impl OutboxEvent for FriendCreatedEvent {
///     fn exchange_name(&self) -> String {
///         "beep.community".to_string()
///     }
///
///     fn routing_key(&self) -> String {
///         "friend.created".to_string()
///     }
/// }
/// ```
pub trait OutboxEvent: Serialize {
    /// Returns the exchange or topic name where this event should be published
    fn exchange_name(&self) -> String;

    /// Returns the routing key for message broker routing
    fn routing_key(&self) -> String;

    /// Optional: Returns a unique event identifier
    ///
    /// Defaults to generating a new UUID. Override if you want to use a specific event ID
    /// from your event payload for idempotency.
    fn event_id(&self) -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }
}
