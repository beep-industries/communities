use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements, services::Service},
    friend::ports::FriendshipRepository,
    health::port::HealthRepository,
    server::{
        entities::{InsertServerInput, Server, ServerId, UpdateServerInput},
        ports::{ServerRepository, ServerService},
    },
};

impl<S, F, H> ServerService for Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    async fn create_server(&self, input: InsertServerInput) -> Result<Server, CoreError> {
        // Validate server name is not empty
        if input.name.trim().is_empty() {
            return Err(CoreError::InvalidServerName);
        }

        // @TODO Authorization: Check if the user has permission to create servers

        // Create the server via repository
        let server = self.server_repository.insert(input).await?;

        Ok(server)
    }

    async fn get_server(&self, server_id: &ServerId) -> Result<Server, CoreError> {
        // @TODO Authorization: Check if the user has permission to access the server

        let server = self.server_repository.find_by_id(server_id).await?;

        match server {
            Some(server) => Ok(server),
            None => Err(CoreError::ServerNotFound {
                id: server_id.clone(),
            }),
        }
    }

    async fn list_servers(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        // @TODO Authorization: Filter servers by visibility based on user permissions

        let (servers, total) = self.server_repository.list(pagination).await?;

        Ok((servers, total))
    }

    async fn update_server(&self, input: UpdateServerInput) -> Result<Server, CoreError> {
        // Check if server exists
        let existing_server = self.server_repository.find_by_id(&input.id).await?;

        if existing_server.is_none() {
            return Err(CoreError::ServerNotFound {
                id: input.id.clone(),
            });
        }

        // Validate name if it's being updated
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(CoreError::InvalidServerName);
            }
        }

        // @TODO Authorization: Verify user is the server owner or has admin privileges

        // Update the server
        let updated_server = self.server_repository.update(input).await?;

        Ok(updated_server)
    }

    async fn delete_server(&self, server_id: &ServerId) -> Result<(), CoreError> {
        // Check if server exists
        let existing_server = self.server_repository.find_by_id(server_id).await?;

        if existing_server.is_none() {
            return Err(CoreError::ServerNotFound {
                id: server_id.clone(),
            });
        }

        // @TODO Authorization: Verify user is the server owner or has admin privileges

        // Delete the server
        self.server_repository.delete(server_id).await?;

        Ok(())
    }
}
