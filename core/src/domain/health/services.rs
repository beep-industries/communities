use crate::domain::{
    common::{CoreError, services::Service},
    friend::ports::FriendshipRepository,
    health::{
        entities::IsHealthy,
        port::{HealthRepository, HealthService},
    },
    server::ports::ServerRepository,
    server_member::ports::MemberRepository,
    channel::ports::ChannelRepository,
};

impl<S, F, H, M, C> HealthService for Service<S, F, H, M, C>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
