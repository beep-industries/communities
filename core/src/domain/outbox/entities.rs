use std::sync::Mutex;

use chrono::{DateTime, Utc};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::outbox::error::OutboxError;

/// Represents an outbox message entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: Uuid,
    pub exchange_name: String,
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

pub struct OutboxMessageStream {
    stream: Mutex<std::pin::Pin<Box<dyn Stream<Item = Result<OutboxMessage, OutboxError>> + Send>>>,
}

impl OutboxMessageStream {
    pub fn new(
        stream: impl Stream<Item = Result<OutboxMessage, OutboxError>> + Send + 'static,
    ) -> Self {
        Self {
            stream: Mutex::new(Box::pin(stream)),
        }
    }
}

impl Stream for OutboxMessageStream {
    type Item = Result<OutboxMessage, OutboxError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.lock().unwrap().as_mut().poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.lock().unwrap().size_hint()
    }
}
