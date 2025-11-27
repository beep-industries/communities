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

#[derive(Clone)]
pub struct CommunitiesRepositories {
    pub server_repository: PostgresServerRepository,
    pub friendship_repository: PostgresFriendshipRepository,
    pub health_repository: PostgresHealthRepository,
}

pub async fn create_repositories(
    pg_connection_options: PgConnectOptions,
) -> Result<CommunitiesRepositories, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let server_repository = PostgresServerRepository::new(pool.clone());
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool.clone());
    Ok(CommunitiesRepositories {
        server_repository,
        friendship_repository,
        health_repository,
    })
}

impl Into<CommunitiesService> for CommunitiesRepositories {
    fn into(self) -> CommunitiesService {
        Service::new(
            self.server_repository,
            self.friendship_repository,
            self.health_repository,
        )
    }
}
