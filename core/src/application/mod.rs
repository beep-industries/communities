use sqlx::PgPool;

use crate::{
    domain::common::services::Service,
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
pub fn create_service(pool: PgPool) -> CommunitiesService {
    let server_repository = PostgresServerRepository::new(pool.clone());
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool);

    Service::new(server_repository, friendship_repository, health_repository)
}
