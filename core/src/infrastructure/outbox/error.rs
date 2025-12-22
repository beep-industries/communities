use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum OutboxError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Outbox event with id {id} not found")]
    EventNotFound { id: uuid::Uuid },

    #[error("Failed to listen to outbox notifications")]
    ListenerError,

    #[error("Failed to serialize or deserialize event data")]
    SerializationError,
}

impl OutboxError {
    pub fn error_code(&self) -> &'static str {
        match self {
            OutboxError::EventNotFound { .. } => "E_OUTBOX_EVENT_NOT_FOUND",
            OutboxError::ListenerError => "E_OUTBOX_LISTENER_ERROR",
            OutboxError::SerializationError => "E_OUTBOX_SERIALIZATION_ERROR",
            _ => "E_UNKNOWN_OUTBOX_ERROR",
        }
    }
}
