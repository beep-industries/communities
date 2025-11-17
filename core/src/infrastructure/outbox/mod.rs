//! Outbox pattern infrastructure for transactional event publishing
//!
//! This module provides the core primitives for implementing the transactional outbox pattern:
//! - `OutboxEvent` trait for defining domain events
//! - `write_event` helper for writing events within database transactions
//! - `OutboxError` for error handling

mod error;
mod event;
mod writer;

pub use error::OutboxError;
pub use event::OutboxEvent;
pub use writer::write_event;
