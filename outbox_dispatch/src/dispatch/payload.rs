use communities_core::{
    application::Routing,
    domain::{
        member_role::entities::{AssignUserRole, MemberRole, UnassignUserRole},
        outbox::entities::OutboxMessage,
        role::entities::{DeleteRole, Role},
        server::entities::{DeleteServerEvent, Server},
        server_member::ServerMember,
    },
};
use events_protobuf::communities_events::{
    self, CreateServer, DeleteServer, MemberAssignedToRole, MemberRemovedFromRole, UpsertRole,
    UserJoinServer, UserLeaveServer,
};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{dispatch::DispatcherError, lapin::ExchangeName};

pub enum ExchangePayload {
    CreateServer(ProcessedEvent<CreateServer, Server>),
    DeleteServer(ProcessedEvent<DeleteServer, DeleteServerEvent>),
    UserJoinServer(ProcessedEvent<UserJoinServer, ServerMember>),
    UserLeaveServer(ProcessedEvent<UserLeaveServer, ServerMember>),
    UpsertRole(ProcessedEvent<UpsertRole, Role>),
    DeleteRole(ProcessedEvent<communities_events::DeleteRole, DeleteRole>),
    MemberAssignToRole(ProcessedEvent<MemberAssignedToRole, AssignUserRole>),
    MemberUnassignFromRole(ProcessedEvent<MemberRemovedFromRole, UnassignUserRole>),
}

impl TryFrom<(OutboxMessage, Routing)> for ExchangePayload {
    type Error = DispatcherError;

    fn try_from((outbox, routing): (OutboxMessage, Routing)) -> Result<Self, Self::Error> {
        let payload = match routing {
            Routing::CreateServer => ExchangePayload::CreateServer(ProcessedEvent::new(outbox)?),
            Routing::CreateChannel => ExchangePayload::CreateServer(ProcessedEvent::new(outbox)?),
            Routing::DeleteServer => ExchangePayload::DeleteServer(ProcessedEvent::new(outbox)?),
            Routing::UserJoinServer => {
                ExchangePayload::UserJoinServer(ProcessedEvent::new(outbox)?)
            }
            Routing::UserLeaveServer => {
                ExchangePayload::UserLeaveServer(ProcessedEvent::new(outbox)?)
            }
            Routing::UpsertRole => ExchangePayload::UpsertRole(ProcessedEvent::new(outbox)?),
            Routing::DeleteRole => ExchangePayload::DeleteRole(ProcessedEvent::new(outbox)?),
            Routing::MemberAssignToRole => {
                ExchangePayload::MemberAssignToRole(ProcessedEvent::new(outbox)?)
            }
            Routing::MemberUnassignFromRole => {
                ExchangePayload::MemberUnassignFromRole(ProcessedEvent::new(outbox)?)
            }
        };
        Ok(payload)
    }
}

impl ExchangePayload {
    pub fn exchange_name(&self) -> &ExchangeName {
        match self {
            ExchangePayload::CreateServer(event) => &event.2,
            ExchangePayload::DeleteServer(event) => &event.2,
            ExchangePayload::UserJoinServer(event) => &event.2,
            ExchangePayload::UserLeaveServer(event) => &event.2,
            ExchangePayload::UpsertRole(event) => &event.2,
            ExchangePayload::DeleteRole(event) => &event.2,
            ExchangePayload::MemberAssignToRole(event) => &event.2,
            ExchangePayload::MemberUnassignFromRole(event) => &event.2,
        }
    }

    pub fn encode_proto(&self) -> Vec<u8> {
        match self {
            ExchangePayload::CreateServer(event) => event.0.encode_to_vec(),
            ExchangePayload::DeleteServer(event) => event.0.encode_to_vec(),
            ExchangePayload::UserJoinServer(event) => event.0.encode_to_vec(),
            ExchangePayload::UserLeaveServer(event) => event.0.encode_to_vec(),
            ExchangePayload::UpsertRole(event) => event.0.encode_to_vec(),
            ExchangePayload::DeleteRole(event) => event.0.encode_to_vec(),
            ExchangePayload::MemberAssignToRole(event) => event.0.encode_to_vec(),
            ExchangePayload::MemberUnassignFromRole(event) => event.0.encode_to_vec(),
        }
    }

    /// Returns `true` if the exchange payload is [`MemberAssignToRole`].
    ///
    /// [`MemberAssignToRole`]: ExchangePayload::MemberAssignToRole
    #[must_use]
    pub fn is_member_assign_to_role(&self) -> bool {
        matches!(self, Self::MemberAssignToRole(..))
    }
}

pub struct ProcessedEvent<
    TProtoMessage: Message,
    TOutboxPayload: Into<TProtoMessage> + for<'a> Deserialize<'a> + Clone,
>(TProtoMessage, TOutboxPayload, ExchangeName);

impl<TProtoMessage, TOutboxPayload> ProcessedEvent<TProtoMessage, TOutboxPayload>
where
    TOutboxPayload: Into<TProtoMessage> + for<'a> Deserialize<'a> + Clone,
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
