use crate::{
    domain::common::CoreError,
    infrastructure::outbox::event::{MessageRouter, OutboxEventRecord},
};
use chrono::Utc;
use serde::Serialize;
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
/// use communities_core::infrastructure::outbox::{write_outbox_event, OutboxEventRecord, MessageRoutingInfo};
/// use serde::Serialize;
///
/// #[derive(Serialize, Clone)]
/// struct MyEvent { data: String }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let pool = PgPool::connect("postgres://postgres:password@localhost:5432/communities").await?;
///     let mut tx = pool.begin().await?;
///
///     // Define routing and event payload
///     let router = MessageRoutingInfo::new("my.exchange");
///     let event = OutboxEventRecord::new(router, MyEvent { data: "test".to_string() });
///
///     // Write event to outbox within the transaction
///     let _event_id = write_outbox_event(&mut *tx, &event).await?;
///     tx.commit().await?;
///     Ok(())
/// }
/// ```
pub async fn write_outbox_event<'e, E, TPayload, TRouter>(
    executor: E,
    event: &OutboxEventRecord<TPayload, TRouter>,
) -> Result<Uuid, CoreError>
where
    E: PgExecutor<'e>,
    TPayload: Serialize,
    TRouter: MessageRouter,
{
    let exchange_name = event.router.exchange_name();
    let created_at = Utc::now();

    // Serialize event to JSON
    let payload = serde_json::to_value(event)
        .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;

    // Insert into outbox_messages table
    let query = r#"
        INSERT INTO outbox_messages (id, exchange_name, payload, status, failed_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO NOTHING
    "#;

    sqlx::query(query)
        .bind(event.id)
        .bind(exchange_name)
        .bind(payload)
        .bind("READY")
        .bind(None::<chrono::DateTime<Utc>>)
        .bind(created_at)
        .execute(executor)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    Ok(event.id)
}
