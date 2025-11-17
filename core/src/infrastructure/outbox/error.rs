use serde_json;
use thiserror::Error;

/// Errors that can occur when working with the outbox
#[derive(Error, Debug)]
pub enum OutboxError {
    /// Database error occurred during outbox operation
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// Serialization error occurred when converting event to JSON
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
