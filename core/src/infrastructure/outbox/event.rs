use serde::Serialize;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{domain::common::CoreError, write_outbox_event};

pub struct OutboxEventRecord<TPayload: Serialize, TRouter: MessageRouter> {
    pub id: Uuid,
    pub router: TRouter,
    pub payload: TPayload,
}

impl<TPayload: Serialize + Clone, TRouter: MessageRouter> OutboxEventRecord<TPayload, TRouter> {
    pub fn new(router: TRouter, payload: TPayload) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            id: uuid,
            router,
            payload,
        }
    }

    pub async fn write(&self, executor: impl PgExecutor<'_>) -> Result<Uuid, CoreError> {
        write_outbox_event(executor, self).await
    }
}

#[derive(Clone)]
pub struct MessageRoutingInfo(ExchangeName, RoutingKey);

pub trait MessageRouter {
    fn exchange_name(&self) -> String;
    fn routing_key(&self) -> String;
}

impl MessageRouter for MessageRoutingInfo {
    fn exchange_name(&self) -> String {
        self.0.clone()
    }
    fn routing_key(&self) -> String {
        self.1.clone()
    }
}
impl<TPayload: Serialize, TRouter: MessageRouter> Serialize
    for OutboxEventRecord<TPayload, TRouter>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.payload.serialize(serializer)
    }
}

pub type ExchangeName = String;
pub type RoutingKey = String;
