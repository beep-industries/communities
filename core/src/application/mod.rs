use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::{
    domain::common::{CoreError, services::Service},
    infrastructure::{
        friend::repositories::postgres::PostgresFriendshipRepository,
        health::repositories::postgres::PostgresHealthRepository,
        server::repositories::postgres::PostgresServerRepository,
    },
};

/// Concrete service type with PostgreSQL repositories
pub type CommunitiesService =
    Service<PostgresServerRepository, PostgresFriendshipRepository, PostgresHealthRepository>;

/// Factory function to create the communities service with all dependencies
pub async fn create_service(
    pg_connection_options: PgConnectOptions,
) -> Result<CommunitiesService, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let server_repository = PostgresServerRepository::new(pool.clone());
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool);
    Ok(Service::new(
        server_repository,
        friendship_repository,
        health_repository,
    ))
}
