use communities_core::{
    application::Routing,
    domain::{outbox::entities::OutboxMessage, server::entities::Server},
};
use events_protobuf::communities_events::CreateServer;
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{dispatch::DispatcherError, lapin::ExchangeName};

impl OutboxPayload<CreateServer> for Server {}

pub enum ExchangePayload {
    CreateServer(ProcessedEvent<CreateServer, Server>),
}

impl TryFrom<(OutboxMessage, Routing)> for ExchangePayload {
    type Error = DispatcherError;

    fn try_from((outbox, routing): (OutboxMessage, Routing)) -> Result<Self, Self::Error> {
        let payload = match routing {
            Routing::CreateServer => ExchangePayload::CreateServer(ProcessedEvent::new(outbox)?),
            Routing::CreateChannel => ExchangePayload::CreateServer(ProcessedEvent::new(outbox)?),
        };
        Ok(payload)
    }
}

impl ExchangePayload {
    pub fn exchange_name(&self) -> &ExchangeName {
        match self {
            ExchangePayload::CreateServer(event) => &event.2,
        }
    }

    pub fn encode_proto(&self) -> Vec<u8> {
        match self {
            ExchangePayload::CreateServer(event) => event.0.encode_to_vec(),
        }
    }
}

pub trait OutboxPayload<TProtoMessage>:
    Into<TProtoMessage> + Serialize + for<'a> Deserialize<'a> + Clone
{
}

pub struct ProcessedEvent<TProtoMessage: Message, TOutboxPayload: OutboxPayload<TProtoMessage>>(
    TProtoMessage,
    TOutboxPayload,
    ExchangeName,
);

impl<TProtoMessage, TOutboxPayload> ProcessedEvent<TProtoMessage, TOutboxPayload>
where
    TOutboxPayload: OutboxPayload<TProtoMessage>,
    TProtoMessage: Message,
{
    pub fn new(outbox_event: OutboxMessage) -> Result<Self, DispatcherError> {
        let raw_payload = outbox_event
            .payload::<TOutboxPayload>()
            .map_err(|e| DispatcherError::WrongPayloadError { msg: e.to_string() })?;
        let proto: TProtoMessage = raw_payload.clone().into();
        Ok(Self(proto, raw_payload, outbox_event.exchange_name))
    }

    pub fn proto(&self) -> &TProtoMessage {
        &self.0
    }

    pub fn payload(&self) -> &TOutboxPayload {
        &self.1
    }

    pub fn exchange_name(&self) -> &ExchangeName {
        &self.2
    }
}
