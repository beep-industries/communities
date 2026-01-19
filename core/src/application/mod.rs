use std::collections::HashMap;

use beep_auth::KeycloakAuthRepository;
use beep_authz::{SpiceDbConfig, SpiceDbRepository};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    domain::{
        channel_member::ports::MockChannelMemberRepository,
        common::{CoreError, services::Service},
    },
    infrastructure::{
        MessageRoutingInfo,
        authorization::SpiceDbAuthorizationRepository,
        channel::repositories::PostgresChannelRepository,
        friend::repositories::postgres::PostgresFriendshipRepository,
        health::repositories::postgres::PostgresHealthRepository,
        member_role::repositories::postgres::PostgresMemberRoleRepository,
        outbox::{MessageRouter, postgres::PostgresOutboxRepository},
        role::repositories::postgres::PostgresRoleRepository,
        server::repositories::postgres::PostgresServerRepository,
        server_invitation::repositories::postgres::PostgresServerInvitationRepository,
        server_member::repositories::PostgresMemberRepository,
        user::repositories::http::HttpUserRepository,
    },
};

pub struct BeepServicesConfig {
    pub user_service_url: String,
}

/// Concrete service type with PostgreSQL repositories
pub type CommunitiesService = Service<
    PostgresServerRepository,
    PostgresFriendshipRepository,
    HttpUserRepository,
    PostgresHealthRepository,
    PostgresMemberRepository,
    PostgresChannelRepository,
    PostgresRoleRepository,
    PostgresOutboxRepository,
    MockChannelMemberRepository,
    PostgresMemberRoleRepository,
    PostgresServerInvitationRepository,
    SpiceDbAuthorizationRepository,
>;

#[derive(Clone)]
pub struct CommunitiesRepositories {
    pool: PgPool,
    pub server_repository: PostgresServerRepository,
    pub friendship_repository: PostgresFriendshipRepository,
    pub user_repository: HttpUserRepository,
    pub health_repository: PostgresHealthRepository,
    pub member_repository: PostgresMemberRepository,
    pub channel_repository: PostgresChannelRepository,
    pub keycloak_repository: KeycloakAuthRepository,
    pub role_repository: PostgresRoleRepository,
    pub outbox_repository: PostgresOutboxRepository,
    pub channel_member_repository: MockChannelMemberRepository,
    pub member_role_repository: PostgresMemberRoleRepository,
    pub server_invitation_repository: PostgresServerInvitationRepository,
    pub authorization_repository: SpiceDbAuthorizationRepository,
}

pub async fn create_repositories(
    pg_connection_options: PgConnectOptions,
    message_routing_config: MessageRoutingConfig,
    keycloak_issuer: String,
    beep_services: BeepServicesConfig,
    spicedb_config: SpiceDbConfig,
) -> Result<CommunitiesRepositories, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let server_repository = PostgresServerRepository::new(
        pool.clone(),
        message_routing_config.clone().delete_server,
        message_routing_config.clone().create_server,
        message_routing_config.clone().upsert_role,
        message_routing_config.clone().user_join_server,
    );
    let friendship_repository = PostgresFriendshipRepository::new(pool.clone());
    let user_repository = HttpUserRepository::new(beep_services.user_service_url);
    let health_repository = PostgresHealthRepository::new(pool.clone());
    let member_repository = PostgresMemberRepository::new(
        pool.clone(),
        message_routing_config.clone().user_leave_server,
        message_routing_config.clone().user_join_server,
        message_routing_config.clone().member_assign_to_role,
    );
    let channel_repository = PostgresChannelRepository::new(
        pool.clone(),
        message_routing_config.clone().create_channel,
        message_routing_config.clone().delete_channel,
    );
    let keycloak_repository = KeycloakAuthRepository::new(keycloak_issuer, None);
    let role_repository = PostgresRoleRepository::new(
        pool.clone(),
        message_routing_config.clone().upsert_role,
        message_routing_config.clone().upsert_role,
        message_routing_config.clone().delete_role,
    );
    let outbox_repository = PostgresOutboxRepository::new(pool.clone());
    let channel_member_repository = MockChannelMemberRepository::new();
    let member_role_repository =
        PostgresMemberRoleRepository::new(pool.clone(), message_routing_config.clone().upsert_role);
    let server_invitation_repository = PostgresServerInvitationRepository::new(pool.clone());
    let spicedb_repository = SpiceDbRepository::new(spicedb_config)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let authorization_repository = SpiceDbAuthorizationRepository::new(spicedb_repository);
    Ok(CommunitiesRepositories {
        pool,
        server_repository,
        health_repository,
        friendship_repository,
        user_repository,
        member_repository,
        channel_repository,
        role_repository,
        keycloak_repository,
        outbox_repository,
        channel_member_repository,
        member_role_repository,
        server_invitation_repository,
        authorization_repository,
    })
}

impl From<CommunitiesRepositories> for CommunitiesService {
    fn from(repos: CommunitiesRepositories) -> Self {
        Service::new(
            repos.server_repository,
            repos.friendship_repository,
            repos.user_repository,
            repos.health_repository,
            repos.member_repository,
            repos.channel_repository,
            repos.role_repository,
            repos.outbox_repository,
            repos.channel_member_repository,
            repos.member_role_repository,
            repos.server_invitation_repository,
            repos.authorization_repository,
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
    pub user_join_server: MessageRoutingInfo,
    pub user_leave_server: MessageRoutingInfo,
    pub upsert_role: MessageRoutingInfo,
    pub delete_role: MessageRoutingInfo,
    pub member_assign_to_role: MessageRoutingInfo,
    pub member_unassign_from_role: MessageRoutingInfo,
}

impl MessageRoutingConfig {
    pub fn from_string_to_routing(&self, value: String) -> Option<Routing> {
        self.to_raw().get(&value).cloned()
    }

    fn to_raw(&self) -> HashMap<String, Routing> {
        let mut config = HashMap::<String, Routing>::new();
        config.insert(self.create_channel.exchange_name(), Routing::CreateChannel);
        config.insert(self.delete_channel.exchange_name(), Routing::DeleteChannel);
        config.insert(self.create_server.exchange_name(), Routing::CreateServer);
        config.insert(self.delete_server.exchange_name(), Routing::DeleteServer);
        config.insert(self.upsert_role.exchange_name(), Routing::UpsertRole);
        config.insert(self.delete_role.exchange_name(), Routing::DeleteRole);
        config.insert(
            self.user_join_server.exchange_name(),
            Routing::UserJoinServer,
        );
        config.insert(
            self.user_leave_server.exchange_name(),
            Routing::UserLeaveServer,
        );

        config.insert(
            self.member_assign_to_role.exchange_name(),
            Routing::MemberAssignToRole,
        );
        config.insert(
            self.member_unassign_from_role.exchange_name(),
            Routing::MemberUnassignFromRole,
        );
        config
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Routing {
    CreateServer,
    DeleteServer,
    CreateChannel,
    DeleteChannel,
    UserJoinServer,
    UserLeaveServer,
    UpsertRole,
    DeleteRole,
    MemberAssignToRole,
    MemberUnassignFromRole,
}
