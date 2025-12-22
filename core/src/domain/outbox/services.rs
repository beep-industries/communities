use futures_util::Stream;
use serde_json::Value;

use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        common::{GetPaginated, TotalPaginatedElements},
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        outbox::{
            entities::{OutboxMessage, OutboxStatus},
            error::OutboxError,
            ports::{OutboxRepository, OutboxService},
        },
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O> OutboxService for Service<S, F, H, M, C, R, O>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
{
    async fn get(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<OutboxMessage>, TotalPaginatedElements), OutboxError> {
        self.outbox_repository.get(pagination).await
    }

    async fn delete_marked(&self) -> Result<u64, OutboxError> {
        self.outbox_repository.delete_marked().await
    }

    async fn mark_event_send(&self, id: uuid::Uuid) -> Result<OutboxMessage, OutboxError> {
        self.outbox_repository
            .mark_event(id, OutboxStatus::Sent)
            .await
    }

    async fn listen_outbox_event(
        &self,
    ) -> Result<impl Stream<Item = Result<Value, OutboxError>>, OutboxError> {
        self.outbox_repository.listen_outbox_event().await
    }
}
