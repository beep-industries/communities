use crate::domain::{
    common::{CoreError, services::Service},
    friend::ports::FriendshipRepository,
    health::port::HealthRepository,
    server::{
        entities::{Server, ServerId},
        ports::{ServerRepository, ServerService},
    },
};

impl<S, F, H> ServerService for Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
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
}
