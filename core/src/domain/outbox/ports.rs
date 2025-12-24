use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::{
    common::{GetPaginated, TotalPaginatedElements},
    outbox::{
        entities::{OutboxMessage, OutboxMessageStream, OutboxStatus},
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
    fn listen_outbox_event(&self)
    -> impl Future<Output = Result<OutboxMessageStream, OutboxError>>;

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
    fn listen_outbox_event(&self)
    -> impl Future<Output = Result<OutboxMessageStream, OutboxError>>;

    fn delete_marked(&self) -> impl Future<Output = Result<u64, OutboxError>>;

    /// Mark event as sent. This is usefull for dispatchers.
    fn mark_event_send(&self, id: Uuid)
    -> impl Future<Output = Result<OutboxMessage, OutboxError>>;
}
pub struct MockOutboxRepository {
    outbox_events: Arc<Mutex<Vec<OutboxMessage>>>,
}

impl MockOutboxRepository {
    pub fn new() -> Self {
        Self {
            outbox_events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl OutboxRepository for MockOutboxRepository {
    async fn get(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<OutboxMessage>, TotalPaginatedElements), OutboxError> {
        let events = self.outbox_events.lock().unwrap();
        let total = events.len() as u64;
        let offset = (pagination.page - 1) * pagination.limit;
        let paginated: Vec<OutboxMessage> = events
            .iter()
            .skip(offset as usize)
            .take(pagination.limit as usize)
            .cloned()
            .collect();
        Ok((paginated, total))
    }

    async fn listen_outbox_event(&self) -> Result<OutboxMessageStream, OutboxError> {
        let events = self.outbox_events.lock().unwrap();
        let all_events: Vec<OutboxMessage> = events.clone();
        let stream = futures_util::stream::iter(all_events.into_iter().map(Ok));
        let message_stream = OutboxMessageStream::new(stream);
        Ok(message_stream)
    }

    async fn delete_marked(&self) -> Result<u64, OutboxError> {
        let mut events = self.outbox_events.lock().unwrap();
        let initial_len = events.len();
        events.retain(|event| event.status != OutboxStatus::Sent);
        Ok((initial_len - events.len()) as u64)
    }

    async fn mark_event(
        &self,
        id: Uuid,
        status: OutboxStatus,
    ) -> Result<OutboxMessage, OutboxError> {
        let mut events = self.outbox_events.lock().unwrap();
        let message = events.iter_mut().find(|message| message.id == id);
        let message = match message {
            Some(message) => message,
            None => return Err(OutboxError::EventNotFound { id }),
        };
        message.status = status;
        Ok(message.to_owned())
    }
}
