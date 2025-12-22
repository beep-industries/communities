//! Outbox pattern infrastructure for transactional event publishing
//!
//! This module provides the core primitives for implementing the transactional outbox pattern:
//! - `OutboxEvent` trait for defining domain events
//! - `write_event` helper for writing events within database transactions
//! - `OutboxError` for error handling

mod event;
mod writer;

pub use event::{MessageRouter, MessageRoutingInfo, OutboxEventRecord};
pub use writer::write_outbox_event;

mod error;
pub mod postgres;

pub use error::OutboxError;
pub use postgres::{OutboxMessage, OutboxStatus, PostgresOutboxRepository};
