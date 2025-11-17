use crate::infrastructure::outbox::{OutboxError, OutboxEvent};
use chrono::Utc;
use serde_json;
use sqlx::PgExecutor;
use uuid::Uuid;

/// Write an event to the outbox table within an existing transaction.
///
/// This function serializes the event to JSONB and inserts it into the `outbox_messages` table
/// with status='READY'. The insert happens within the provided executor/transaction, ensuring
/// atomicity with your business logic writes.
///
/// # Arguments
///
/// * `executor` - A SQLx Postgres executor (transaction or pool)
/// * `event` - The event to write, must implement `OutboxEvent` and `Serialize`
///
/// # Returns
///
/// The UUID of the inserted outbox message on success, or an `OutboxError` on failure.
///
/// # Example
///
/// ```rust,no_run
/// use sqlx::PgPool;
/// use core::infrastructure::outbox::{write_event, OutboxEvent};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyEvent {
///     data: String,
/// }
///
/// impl OutboxEvent for MyEvent {
///     fn exchange_name(&self) -> String { "my.exchange".to_string() }
///     fn routing_key(&self) -> String { "my.key".to_string() }
/// }
///
/// async fn example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
///     let mut tx = pool.begin().await?;
///     
///     // Your business logic here...
///     
///     // Write event to outbox
///     let event = MyEvent { data: "test".to_string() };
///     let event_id = write_event(&mut tx, &event).await?;
///     
///     tx.commit().await?;
///     Ok(())
/// }
/// ```
pub async fn write_event<'e, E, T>(executor: E, event: &T) -> Result<Uuid, OutboxError>
where
    E: PgExecutor<'e>,
    T: OutboxEvent,
{
    let event_id = event.event_id();
    let exchange_name = event.exchange_name();
    let routing_key = event.routing_key();
    let created_at = Utc::now();

    // Serialize event to JSON
    let payload = serde_json::to_value(event)?;

    // Insert into outbox_messages table
    let query = r#"
        INSERT INTO outbox_messages (id, exchange_name, routing_key, payload, status, failed_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO NOTHING
    "#;

    sqlx::query(query)
        .bind(event_id)
        .bind(exchange_name)
        .bind(routing_key)
        .bind(payload)
        .bind("READY")
        .bind(None::<chrono::DateTime<Utc>>)
        .bind(created_at)
        .execute(executor)
        .await?;

    Ok(event_id)
}
