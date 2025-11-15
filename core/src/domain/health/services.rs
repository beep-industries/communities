use crate::domain::{
    common::{CoreError, services::Service},
    friend::ports::FriendshipRepository,
    health::{
        entities::IsHealthy,
        port::{HealthRepository, HealthService},
    },
    server::ports::ServerRepository,
};

impl<S, F, H> HealthService for Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
