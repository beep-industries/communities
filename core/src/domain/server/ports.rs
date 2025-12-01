use std::sync::{Arc, Mutex};

use crate::domain::{
    common::CoreError,
    server::entities::{InsertServerInput, Server, ServerId, UpdateServerInput},
};

pub trait ServerRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertServerInput,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;
    fn find_by_id(
        &self,
        id: &ServerId,
    ) -> impl Future<Output = Result<Option<Server>, CoreError>> + Send;
    fn update(
        &self,
        input: UpdateServerInput,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;
    fn delete(&self, id: &ServerId) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// A service for managing server operations in the application.
///
/// This trait defines the core business logic operations that can be performed on servers.
/// It follows the ports and adapters pattern, where this trait acts as a port that defines
/// the interface for server-related operations. Implementations of this trait will provide
/// the actual business logic while maintaining separation of concerns.
///
/// The trait requires `Send + Sync` to ensure thread safety in async contexts, making it
/// suitable for use in web servers and other concurrent applications
///
/// # Thread Safety
///
/// All implementations must be thread-safe (`Send + Sync`) to support concurrent access
/// in multi-threaded environments.
pub trait ServerService: Send + Sync {
    /// Retrieves a server by its unique identifier.
    ///
    /// This meethod performs the core business logic for fetching a server, including
    /// any necessary authorization checks and data validation. The implementation
    /// should handle cases where the server doesn't exist gracefully.
    ///
    /// # Arguments
    ///
    /// * `server_id` - A reference to the unique identifier of the server to retrieve.
    ///   This should be a valid [`ServerId`] that represents an existing server.
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(Server)` - The server was found and the user has permission to access it
    /// - `Err(CoreError::ServerNotFound)` - No server exists with the given ID
    /// - `Err(CoreError)` - Other errors such as database connectivity issues or authorization failures
    fn get_server(
        &self,
        server_id: &ServerId,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;
}

pub struct MockServerRepository {
    servers: Arc<Mutex<Vec<Server>>>,
}

impl MockServerRepository {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ServerRepository for MockServerRepository {
    async fn find_by_id(&self, id: &ServerId) -> Result<Option<Server>, CoreError> {
        let servers = self.servers.lock().unwrap();

        let server = servers.iter().find(|s| &s.id == id).cloned();

        Ok(server)
    }

    async fn insert(&self, input: InsertServerInput) -> Result<Server, CoreError> {
        let mut servers = self.servers.lock().unwrap();

        let new_server = Server {
            id: ServerId::from(uuid::Uuid::new_v4()),
            name: input.name,
            banner_url: input.banner_url,
            picture_url: input.picture_url,
            description: input.description,
            owner_id: input.owner_id,
            visibility: input.visibility,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        servers.push(new_server.clone());

        Ok(new_server)
    }

    async fn update(&self, input: UpdateServerInput) -> Result<Server, CoreError> {
        let mut servers = self.servers.lock().unwrap();

        let server = servers
            .iter_mut()
            .find(|s| &s.id == &input.id)
            .ok_or_else(|| CoreError::ServerNotFound { id: input.id.clone() })?;

        if let Some(name) = input.name {
            server.name = name;
        }
        if let Some(picture_url) = input.picture_url {
            server.picture_url = Some(picture_url);
        }
        if let Some(banner_url) = input.banner_url {
            server.banner_url = Some(banner_url);
        }
        if let Some(description) = input.description {
            server.description = Some(description);
        }
        if let Some(visibility) = input.visibility {
            server.visibility = visibility;
        }
        server.updated_at = Some(chrono::Utc::now());

        Ok(server.clone())
    }

    async fn delete(&self, id: &ServerId) -> Result<(), CoreError> {
        let mut servers = self.servers.lock().unwrap();

        let index = servers
            .iter()
            .position(|s| &s.id == id)
            .ok_or_else(|| CoreError::ServerNotFound { id: id.clone() })?;

        servers.remove(index);

        Ok(())
    }
}
