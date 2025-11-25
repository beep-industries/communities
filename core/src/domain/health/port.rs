use crate::domain::{common::CoreError, health::entities::IsHealthy};
use std::future::Future;

pub trait HealthRepository: Send + Sync {
    fn ping(&self) -> impl Future<Output = IsHealthy> + Send;
}

pub trait HealthService: Send + Sync {
    fn check_health(&self) -> impl Future<Output = Result<IsHealthy, CoreError>> + Send;
}
pub struct MockHealthRepository;

impl MockHealthRepository {
    pub fn new() -> Self {
        Self
    }
}

impl HealthRepository for MockHealthRepository {
    async fn ping(&self) -> IsHealthy {
        IsHealthy::new(true)
    }
}
