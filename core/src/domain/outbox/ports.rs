use futures_util::Stream;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{
    common::{GetPaginated, TotalPaginatedElements},
    outbox::{
        entities::{OutboxMessage, OutboxStatus},
        error::OutboxError,
    },
};

pub trait OutboxRepository: Send + Sync {
    fn get(
        &self,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<OutboxMessage>, TotalPaginatedElements), OutboxError>>;

    /// Return a stream of serializable values that
    /// represente the modification inside the outbox table
    fn listen_outbox_event(
        &self,
    ) -> impl Future<Output = Result<impl Stream<Item = Result<Value, OutboxError>>, OutboxError>>;

    fn delete_marked(&self) -> impl Future<Output = Result<u64, OutboxError>>;

    fn mark_event(
        &self,
        id: Uuid,
        status: OutboxStatus,
    ) -> impl Future<Output = Result<OutboxMessage, OutboxError>>;
}

pub trait OutboxService {
    fn get(
        &self,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<OutboxMessage>, TotalPaginatedElements), OutboxError>>;

    /// Return a stream of serializable values that
    /// represente the modification inside the outbox table
    fn listen_outbox_event(
        &self,
    ) -> impl Future<Output = Result<impl Stream<Item = Result<Value, OutboxError>>, OutboxError>>;

    fn delete_marked(&self) -> impl Future<Output = Result<u64, OutboxError>>;

    /// Mark event as sent. This is usefull for dispatchers.
    fn mark_event_send(&self, id: Uuid)
    -> impl Future<Output = Result<OutboxMessage, OutboxError>>;
}
