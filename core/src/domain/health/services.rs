use crate::domain::{
    common::{CoreError, services::Service},
    health::{entities::IsHealthy, port::{HealthRepository, HealthService}},
    server::ports::ServerRepository,
};

impl<S, H> HealthService for Service<S, H>
where
    S: ServerRepository,
    H: HealthRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
