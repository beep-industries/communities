use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::{GetPaginated, TotalPaginatedElements},
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        outbox::{
            entities::{OutboxMessage, OutboxMessageStream, OutboxStatus},
            error::OutboxError,
            ports::{OutboxRepository, OutboxService},
        },
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O, CM> OutboxService for Service<S, F, H, M, C, R, O, CM>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
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

    async fn listen_outbox_event(&self) -> Result<OutboxMessageStream, OutboxError> {
        self.outbox_repository.listen_outbox_event().await
    }
}
