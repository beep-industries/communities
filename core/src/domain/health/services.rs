use crate::domain::{
    channel::ports::ChannelRepository,
    common::{CoreError, services::Service},
    friend::ports::FriendshipRepository,
    health::{
        entities::IsHealthy,
        port::{HealthRepository, HealthService},
    },
    outbox::ports::OutboxRepository,
    role::ports::RoleRepository,
    server::ports::ServerRepository,
    server_member::ports::MemberRepository,
};

impl<S, F, H, M, C, R, O> HealthService for Service<S, F, H, M, C, R, O>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
