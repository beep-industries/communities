use crate::{
    Service,
    domain::{
        authorization::ports::AuthorizationRepository,
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::{GetPaginated, TotalPaginatedElements},
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::{
            entities::{OutboxMessage, OutboxMessageStream, OutboxStatus},
            error::OutboxError,
            ports::{OutboxRepository, OutboxService},
        },
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_invitation::ports::ServerInvitationRepository,
        server_member::MemberRepository,
        user::port::UserRepository,
    },
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A> OutboxService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI, A>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
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
