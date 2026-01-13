use std::collections::{HashMap, HashSet};

use beep_auth::KeycloakAuthRepository;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    domain::{
        channel_member::ports::MockChannelMemberRepository,
        common::{CoreError, services::Service},
        role::ports::MockRoleRepository,
    },
    infrastructure::{
        MessageRoutingInfo, channel::repositories::PostgresChannelRepository,
        friend::repositories::postgres::PostgresFriendshipRepository,
        health::repositories::postgres::PostgresHealthRepository, outbox::MessageRouter,
        outbox::postgres::PostgresOutboxRepository,
        role::repositories::postgres::PostgresRoleRepository,
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
    PostgresRoleRepository,
    PostgresOutboxRepository,
    PostgresMemberRepository,
>;

#[derive(Clone)]
pub struct CommunitiesRepositories {
    pool: PgPool,
    pub server_repository: PostgresServerRepository,
    pub friendship_repository: PostgresFriendshipRepository,
    pub health_repository: PostgresHealthRepository,
    pub member_repository: PostgresMemberRepository,
    pub channel_repository: PostgresChannelRepository,
    pub keycloak_repository: KeycloakAuthRepository,
    pub role_repository: PostgresRoleRepository,
    pub outbox_repository: PostgresOutboxRepository,
    pub channel_member_repository: PostgresMemberRepository,
}

pub async fn create_repositories(
    pg_connection_options: PgConnectOptions,
    message_routing_config: MessageRoutingConfig,
    keycloak_issuer: String,
) -> Result<CommunitiesRepositories, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let server_repository = PostgresServerRepository::new(
        pool.clone(),
        message_routing_config.delete_server,
        message_routing_config.create_server,
        message_routing_config.create_role,
    );
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let health_repository = PostgresHealthRepository::new(pool.clone());
    let member_repository =
        PostgresMemberRepository::new(pool.clone(), MessageRoutingInfo::default());
    let channel_repository = PostgresChannelRepository::new(
        pool.clone(),
        message_routing_config.create_channel,
        message_routing_config.delete_channel,
    );
    let keycloak_repository = KeycloakAuthRepository::new(keycloak_issuer, None);
    let role_repository = MockRoleRepository::new();
    let outbox_repository = PostgresOutboxRepository::new(pool.clone());
    let channel_member_repository = MockChannelMemberRepository::new();
    Ok(CommunitiesRepositories {
        pool,
        server_repository,
        friendship_repository,
        health_repository,
        member_repository,
        channel_repository,
        role_repository,
        keycloak_repository,
        outbox_repository,
        channel_member_repository,
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
            self.outbox_repository,
            self.channel_member_repository,
            self.member_role_repository,
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
pub struct MessageRoutingConfig {
    /// Routing information for server creation events
    pub create_server: MessageRoutingInfo,
    pub delete_server: MessageRoutingInfo,
    pub create_channel: MessageRoutingInfo,
    pub delete_channel: MessageRoutingInfo,
    pub create_role: MessageRoutingInfo,
}

impl MessageRoutingConfig {
    pub fn from_string_to_routing(&self, value: String) -> Option<Routing> {
        self.to_raw().get(&value).cloned()
    }

    fn to_raw(&self) -> HashMap<String, Routing> {
        let mut config = HashMap::<String, Routing>::new();
        // config.insert(self.create_channel.exchange_name(), Routing::CreateChannel);
        // config.insert(self.delete_channel.exchange_name(), Routing::DeleteChannel);
        config.insert(self.create_server.exchange_name(), Routing::CreateServer);
        // config.insert(self.delete_server.exchange_name(), Routing::DeleteServer);
        config
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Routing {
    CreateServer,
    // DeleteServer,
    // CreateChannel,
    // DeleteChannel,
}
