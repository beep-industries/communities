use beep_auth::KeycloakAuthRepository;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    domain::{
        common::{CoreError, services::Service},
        role::ports::MockRoleRepository,
    },
    infrastructure::{
        MessageRoutingInfo, channel::repositories::PostgresChannelRepository,
        friend::repositories::postgres::PostgresFriendshipRepository,
        health::repositories::postgres::PostgresHealthRepository,
        server::repositories::postgres::PostgresServerRepository,
        server_member::repositories::PostgresMemberRepository,
    },
};

/// Concrete service type with PostgreSQL repositories
pub type CommunitiesService = Service<
    PostgresServerRepository,
    PostgresFriendshipRepository,
    PostgresHealthRepository,
    PostgresMemberRepository,
    PostgresChannelRepository,
    MockRoleRepository,
>;

#[derive(Clone)]
pub struct CommunitiesRepositories {
    pool: PgPool,
    pub server_repository: PostgresServerRepository,
    pub friendship_repository: PostgresFriendshipRepository,
    pub health_repository: PostgresHealthRepository,
    pub member_repository: PostgresMemberRepository,
    pub channel_repository: PostgresChannelRepository,
    pub role_repository: MockRoleRepository,
    pub keycloak_repository: KeycloakAuthRepository,
}

pub async fn create_repositories(
    pg_connection_options: PgConnectOptions,
    message_routing_infos: MessageRoutingInfos,
    keycloak_issuer: String,
) -> Result<CommunitiesRepositories, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let server_repository = PostgresServerRepository::new(
        pool.clone(),
        message_routing_infos.delete_server,
        message_routing_infos.create_server,
    );
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool.clone());
    let member_repository =
        PostgresMemberRepository::new(pool.clone(), MessageRoutingInfo::default());
    let channel_repository = PostgresChannelRepository::new(
        pool.clone(),
        message_routing_infos.create_channel,
        message_routing_infos.delete_channel,
    );
    let keycloak_repository = KeycloakAuthRepository::new(keycloak_issuer, None);
    let role_repository = MockRoleRepository::new();
    Ok(CommunitiesRepositories {
        pool,
        server_repository,
        friendship_repository,
        health_repository,
        member_repository,
        channel_repository,
        role_repository,
        keycloak_repository,
    })
}

impl Into<CommunitiesService> for CommunitiesRepositories {
    fn into(self) -> CommunitiesService {
        Service::new(
            self.server_repository,
            self.friendship_repository,
            self.health_repository,
            self.member_repository,
            self.channel_repository,
            self.role_repository,
        )
    }
}

impl CommunitiesRepositories {
    pub async fn shutdown_pool(&self) {
        let _ = &self.pool.close().await;
    }
}

impl CommunitiesService {
    pub async fn shutdown_pool(&self) {
        self.server_repository.pool.close().await;
    }
}

/// Configuration for message routing information across different event types.
///
/// This struct holds the routing configuration for various outbox events
/// that need to be published to a message broker. Each field represents
/// the routing information (exchange name and routing key) for a specific
/// type of domain event.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MessageRoutingInfos {
    /// Routing information for server creation events
    pub create_server: MessageRoutingInfo,
    /// Routing information for server deletion events
    pub delete_server: MessageRoutingInfo,
    /// Routing information for channel creation events
    pub create_channel: MessageRoutingInfo,
    /// Routing information for channel deletion events
    pub delete_channel: MessageRoutingInfo,
}
