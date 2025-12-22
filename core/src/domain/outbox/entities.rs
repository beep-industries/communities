use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Status of an outbox message
#[derive(Debug, Clone, sqlx::Type, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "VARCHAR", rename_all = "SCREAMING_SNAKE_CASE")]
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
