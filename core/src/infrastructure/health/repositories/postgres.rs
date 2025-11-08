use sqlx::PgPool;

use crate::domain::health::{entities::IsHealthy, port::HealthRepository};

#[derive(Clone)]
pub struct PostgresHealthRepository {
    pool: PgPool,
}

impl PostgresHealthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HealthRepository for PostgresHealthRepository {
    async fn ping(&self) -> IsHealthy {
        IsHealthy::new(
            sqlx::query!("SELECT 1 as health_check")
                .fetch_one(&self.pool)
                .await
                .is_err(),
        )
    }
}
