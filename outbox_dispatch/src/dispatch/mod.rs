use communities_core::{
    application::MessageRoutingConfig,
    domain::outbox::entities::{OutboxMessage, OutboxMessageStream},
};
use futures_util::{Stream, StreamExt, TryStreamExt};
use std::future::Future;
pub mod convert_payload;
pub mod payload;
use crate::{dispatch::payload::ExchangePayload, lapin::RabbitClient};

pub trait PayloadStream:
    Stream<Item = Result<ExchangePayload, DispatcherError>> + Send + Sync + Unpin + 'static
{
}

impl<S> PayloadStream for S where
    S: Stream<Item = Result<ExchangePayload, DispatcherError>> + Send + Sync + Unpin + 'static
{
}

pub struct Dispatcher {
    rabbit_client: RabbitClient,
    outbox_message_stream: Box<dyn PayloadStream>,
    routing: MessageRoutingConfig,
}

impl Dispatcher {
    pub fn new(
        outbox_message_stream: OutboxMessageStream,
        routing: MessageRoutingConfig,
        rabbit_client: RabbitClient,
    ) -> Self {
        let routing_clone = routing.clone();
        let outbox_message_stream = outbox_message_stream
            .map(move |message| -> Result<ExchangePayload, DispatcherError> {
                let message =
                    message.map_err(|e| DispatcherError::MessageError { msg: e.to_string() })?;
                let exchange_name = message.exchange_name.clone();
                let routing = routing_clone
                    .from_string_to_routing(exchange_name.clone())
                    .ok_or_else(|| DispatcherError::WrongExchangeError { exchange_name })?;
                let payload = ExchangePayload::try_from((message, routing))?;
                Ok(payload)
            })
            .into_stream();

        Self {
            rabbit_client,
            outbox_message_stream: Box::new(outbox_message_stream),
            routing,
        }
    }

    async fn send_message(&self, exchange_payload: ExchangePayload) -> Result<(), DispatcherError> {
        println!("{:?}", exchange_payload);
        let encoded = exchange_payload.encode_proto();
        self.rabbit_client
            .produce(exchange_payload.exchange_name(), &encoded)
            .await
            .map_err(|e| DispatcherError::SendMessageError {
                reason: e.to_string(),
            })?;
        Ok(())
    }
}

impl Dispatch for Dispatcher {
    async fn dispatch(&mut self) -> Result<(), std::io::Error> {
        while let Some(stream_message) = self.outbox_message_stream.next().await
            && let Ok(exchange_payload) = stream_message
        {
            let _ = self.send_message(exchange_payload).await;
        }
        Ok(())
    }
}

pub trait Dispatch: Send + Sync {
    fn dispatch(&mut self) -> impl Future<Output = Result<(), std::io::Error>>;
}

#[derive(Debug, thiserror::Error)]
pub enum DispatcherError {
    #[error("The exchange {exchange_name} is not available ")]
    WrongExchangeError { exchange_name: String },

    #[error("Could not send messsage: {reason}")]
    SendMessageError { reason: String },

    #[error("The return payload cannot be handled: {msg}")]
    WrongPayloadError { msg: String },

    #[error("The message can't be processed: {msg}")]
    MessageError { msg: String },
}
