use communities_core::{
    application::{MessageRoutingConfig, Routing},
    domain::{
        outbox::entities::{OutboxMessage, OutboxMessageStream},
        server::entities::Server,
    },
};
use events_protobuf::communities_events::CreateServer;
use futures_util::StreamExt;
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::lapin::RabbitClient;

pub struct Dispatcher {
    rabbit_client: RabbitClient,
    outbox_message_stream: OutboxMessageStream,
    routing: MessageRoutingConfig,
}

impl Dispatcher {
    pub fn new(
        outbox_message_stream: OutboxMessageStream,
        routing: MessageRoutingConfig,
        rabbit_client: RabbitClient,
    ) -> Self {
        Self {
            rabbit_client,
            outbox_message_stream,
            routing,
        }
    }

    async fn send_message<TInput, TPayload>(
        &self,
        input: Value,
        exchange: String,
    ) -> Result<(), DispatcherError>
    where
        TInput: Into<TPayload> + Serialize + for<'a> Deserialize<'a>,
        TPayload: Message,
    {
        let raw_payload: TInput = serde_json::from_value(input).unwrap();
        let payload: TPayload = raw_payload.into();
        self.rabbit_client
            .produce(&exchange, payload)
            .await
            .map_err(|e| DispatcherError::SendMessageError {
                reason: e.to_string(),
            })?;
        Ok(())
    }

    pub async fn handler(&self, payload: OutboxMessage) -> Result<(), DispatcherError> {
        let routing = self
            .routing
            .from_string_to_routing(payload.exchange_name.clone())
            .ok_or_else(|| DispatcherError::WrongExchangeError {
                exchange_name: payload.exchange_name.clone(),
            })?;
        match routing {
            Routing::CreateServer => {
                self.send_message::<Server, CreateServer>(payload.payload, payload.exchange_name)
                    .await?;
            }
        };
        Ok(())
    }
}

impl Dispatch for Dispatcher {
    async fn dispatch(&mut self) -> Result<(), std::io::Error> {
        while let Some(stream_message) = self.outbox_message_stream.next().await
            && let Ok(outbox_message) = stream_message
        {
            let _ = self.handler(outbox_message).await;
        }
        Ok(())
    }
}

pub trait Dispatch: Send + Sync {
    fn dispatch(&mut self) -> impl Future<Output = Result<(), std::io::Error>>;
}

pub enum ExchangePayload {
    CreateServer(Server),
}

#[derive(Debug, thiserror::Error)]
pub enum DispatcherError {
    #[error("The exchange {exchange_name} is not available ")]
    WrongExchangeError { exchange_name: String },

    #[error("Could not send messsage: {reason}")]
    SendMessageError { reason: String },
}
