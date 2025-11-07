use crate::domain::{
    common::{CoreError, services::Service}, friend::ports::{FriendRepository, FriendRequestRepository}, server::{
        entities::{Server, ServerId},
        ports::{ServerRepository, ServerService},
    }
};

impl<S, F, FR> ServerService for Service<S, F, FR>
where
    S: ServerRepository,
    F: FriendRepository,
    FR: FriendRequestRepository,
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
