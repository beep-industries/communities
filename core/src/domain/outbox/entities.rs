use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::outbox::error::OutboxError;

/// Represents an outbox message entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: Uuid,
    pub exchange_name: String,
    pub routing_key: String,
    pub payload: serde_json::Value,
    pub status: OutboxStatus,
    pub failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl OutboxMessage {
    pub fn payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, OutboxError> {
        let payload: T = serde_json::from_value(self.payload.clone())
            .map_err(|_e| OutboxError::SerializationError)?;
        Ok(payload)
    }
}

/// Status of an outbox message
#[derive(Debug, Clone, sqlx::Type, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "VARCHAR", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutboxStatus {
    Ready,
    Sent,
}

impl OutboxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboxStatus::Ready => "READY",
            OutboxStatus::Sent => "SENT",
        }
    }
}
