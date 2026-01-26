use crate::domain::{
    authorization::ports::AuthorizationRepository,
    channel::ports::ChannelRepository,
    channel_member::ports::ChannelMemberRepository,
    common::{CoreError, GetPaginated, TotalPaginatedElements, services::Service},
    friend::{entities::UserId, ports::FriendshipRepository},
    health::port::HealthRepository,
    member_role::ports::MemberRoleRepository,
    outbox::ports::OutboxRepository,
    role::ports::RoleRepository,
    server::{
        entities::{InsertServerInput, Server, ServerId, UpdateServerInput},
        ports::{ServerRepository, ServerService},
    },
    server_invitation::ports::ServerInvitationRepository,
    server_member::MemberRepository,
    server_pictures::ServerPicturesRepository,
    user::port::UserRepository,
};

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> ServerService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
    SC: ServerPicturesRepository,
{
    async fn create_server(&self, input: InsertServerInput) -> Result<Server, CoreError> {
        if input.name.trim().is_empty() {
            return Err(CoreError::InvalidServerName);
        }

        let mut server = self.server_repository.insert(input).await?;

        match self.server_pictures_repository.put_all(server.id).await {
            Ok(server_urls) => {
                server.banner_url = Some(server_urls.banner.to_string());
                server.picture_url = Some(server_urls.picture.to_string());
            }
            Err(e) => tracing::error!("{}", e.to_string()),
        }

        Ok(server)
    }

    async fn get_server(&self, server_id: &ServerId) -> Result<Server, CoreError> {
        let mut server = self.server_repository.find_by_id(server_id).await?;
        match self.server_pictures_repository.get_all(server.id).await {
            Ok(server_urls) => {
                server.banner_url = Some(server_urls.banner.to_string());
                server.picture_url = Some(server_urls.picture.to_string());
            }
            Err(e) => tracing::error!("{}", e.to_string()),
        }
        Ok(server)
    }

    async fn list_servers(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        let (mut servers, total) = self.server_repository.list(pagination).await?;

        let server_ids = servers.iter().map(|server| return server.id).collect();
        let server_urls_map = self
            .server_pictures_repository
            .get_all_for_servers(server_ids)
            .await;

        let servers: Vec<Server> = servers
            .iter_mut()
            .map(|server| {
                if let Some(server_urls) = server_urls_map.get(&server.id) {
                    server.banner_url = Some(server_urls.banner.to_string());
                    server.picture_url = Some(server_urls.picture.to_string());
                }
                return server.to_owned();
            })
            .collect();

        Ok((servers, total))
    }

    async fn update_server(&self, input: UpdateServerInput) -> Result<Server, CoreError> {
        // Validate name if it's being updated
        if let Some(ref name) = input.name
            && name.trim().is_empty()
        {
            return Err(CoreError::InvalidServerName);
        }

        let mut updated_server = self.server_repository.update(input).await?;

        match self
            .server_pictures_repository
            .put_all(updated_server.id)
            .await
        {
            Ok(server_urls) => {
                updated_server.banner_url = Some(server_urls.banner.to_string());
                updated_server.picture_url = Some(server_urls.picture.to_string());
            }
            Err(e) => tracing::error!("{}", e.to_string()),
        }

        Ok(updated_server)
    }

    async fn delete_server(&self, server_id: &ServerId) -> Result<(), CoreError> {
        self.server_repository.delete(server_id).await?;

        Ok(())
    }

    async fn list_user_servers(
        &self,
        pagination: &GetPaginated,
        user_id: UserId,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        let (mut servers, total) = self
            .server_repository
            .list_user_servers(pagination, user_id)
            .await?;

        let server_ids = servers.iter().map(|server| return server.id).collect();
        let server_urls_map = self
            .server_pictures_repository
            .get_all_for_servers(server_ids)
            .await;

        let servers: Vec<Server> = servers
            .iter_mut()
            .map(|server| {
                if let Some(server_urls) = server_urls_map.get(&server.id) {
                    server.banner_url = Some(server_urls.banner.to_string());
                    server.picture_url = Some(server_urls.picture.to_string());
                }
                return server.to_owned();
            })
            .collect();

        Ok((servers, total))
    }

    async fn search_or_discover(
        &self,
        query: Option<String>,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        let (mut servers, total) = self
            .server_repository
            .search_or_discover(query, pagination)
            .await?;

        let server_ids = servers.iter().map(|server| return server.id).collect();
        let server_urls_map = self
            .server_pictures_repository
            .get_all_for_servers(server_ids)
            .await;

        let servers: Vec<Server> = servers
            .iter_mut()
            .map(|server| {
                if let Some(server_urls) = server_urls_map.get(&server.id) {
                    server.banner_url = Some(server_urls.banner.to_string());
                    server.picture_url = Some(server_urls.picture.to_string());
                }
                return server.to_owned();
            })
            .collect();

        Ok((servers, total))
    }
}
