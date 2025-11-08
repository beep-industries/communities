use crate::domain::{common::CoreError, health::entities::IsHealthy};

pub trait HealthRepository: Send + Sync {
    fn ping(&self) -> impl Future<Output = IsHealthy>;
}

pub trait HealthService: Send + Sync {
    fn check_health(&self) -> impl Future<Output = Result<IsHealthy, CoreError>>;
}
