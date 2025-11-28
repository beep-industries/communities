//! Application layer wiring for Communities services.
//!
//! This module assembles the concrete `Service` by configuring the
//! PostgreSQL connection pool and instantiating the repositories used
//! by the domain.
//!
//! Keeping construction concerns outside of pure domain code provides a
//! single entry point to obtain a ready-to-use `CommunitiesService`.
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::{
    domain::common::{CoreError, services::Service},
    infrastructure::{
        MessageRoutingInfo, friend::repositories::postgres::PostgresFriendshipRepository,
        health::repositories::postgres::PostgresHealthRepository,
        server::repositories::postgres::PostgresServerRepository,
    },
};

/// Concrete service type with PostgreSQL repositories
pub type CommunitiesService =
    Service<PostgresServerRepository, PostgresFriendshipRepository, PostgresHealthRepository>;

/// Factory function to create the communities service with all dependencies
///
/// Builds a `sqlx` PostgreSQL pool using the provided connection options,
/// then constructs repository instances and returns a fully-initialized
/// [`CommunitiesService`].
///
/// # Errors
///
/// Returns `CoreError::ServiceUnavailable` if the PostgreSQL pool cannot
/// be created or if any initialization step fails.
///
/// # Parameters
///
/// - `pg_connection_options`: Connection options used to create the `sqlx` pool.
/// - `message_routing_infos`: Routing metadata used by the server repository to
///   emit messages for create/delete operations.
///
/// # Examples
///
/// ```rust
/// use sqlx::postgres::PgConnectOptions;
/// use communities_core::application::{create_service, MessageRoutingInfos};
///
/// # async fn demo() -> Result<(), communities_core::domain::common::CoreError> {
/// let opts = PgConnectOptions::new()
///     .host("localhost")
///     .username("postgres")
///     .password("postgres")
///     .database("communities");
///
/// let routing = MessageRoutingInfos::default();
/// let service = create_service(opts, routing).await?;
/// // use `service`...
/// # Ok(())
/// # }
/// ```
pub async fn create_service(
    pg_connection_options: PgConnectOptions,
    message_routing_infos: MessageRoutingInfos,
) -> Result<CommunitiesService, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let server_repository = PostgresServerRepository::new(
        pool.clone(),
        message_routing_infos.create_server,
        message_routing_infos.delete_server,
    );
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool);
    Ok(Service::new(
        server_repository,
        friendship_repository,
        health_repository,
    ))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
/// Routing metadata required by repositories to publish messages.
///
/// These values are passed to `PostgresServerRepository` so it can
/// emit outbox records (or other message bus integrations) for the
/// corresponding operations.
pub struct MessageRoutingInfos {
    /// Routing information used when creating a server entity.
    pub create_server: MessageRoutingInfo,
    /// Routing information used when deleting a server entity.
    pub delete_server: MessageRoutingInfo,
}
