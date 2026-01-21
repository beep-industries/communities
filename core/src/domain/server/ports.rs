use std::sync::{Arc, Mutex};

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    friend::entities::UserId,
    server::entities::{InsertServerInput, Server, ServerId, UpdateServerInput},
};

pub trait ServerRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertServerInput,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;
    fn find_by_id(&self, id: &ServerId) -> impl Future<Output = Result<Server, CoreError>> + Send;
    fn list(
        &self,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<Server>, TotalPaginatedElements), CoreError>> + Send;
    fn list_user_servers(
        &self,
        pagination: &GetPaginated,
        user_id: UserId,
    ) -> impl Future<Output = Result<(Vec<Server>, TotalPaginatedElements), CoreError>> + Send;
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
    /// Creates a new server with the provided input.
    ///
    /// This method performs business logic validation before delegating to the repository.
    /// It ensures that all required fields are present and valid, and that the user
    /// creating the server has the necessary permissions.
    ///
    /// # Arguments
    ///
    /// * `input` - The server creation input containing name, owner_id, and optional fields
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(Server)` - The newly created server
    /// - `Err(CoreError)` - If validation fails or repository operation fails
    fn create_server(
        &self,
        input: InsertServerInput,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;

    /// Retrieves a server by its unique identifier.
    ///
    /// This method performs the core business logic for fetching a server, including
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

    /// Lists servers with pagination support.
    ///
    /// This method retrieves a paginated list of servers. The implementation should
    /// apply visibility filters based on user permissions and authorization rules.
    ///
    /// # Arguments
    ///
    /// * `pagination` - Pagination parameters (page and limit)
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok((Vec<Server>, TotalPaginatedElements))` - List of servers and total count
    /// - `Err(CoreError)` - If repository operation fails
    fn list_servers(
        &self,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<Server>, TotalPaginatedElements), CoreError>> + Send;

    /// Updates an existing server with the provided input.
    ///
    /// This method validates that the server exists and that the user has permission
    /// to update it before applying the changes. Only non-None fields in the input
    /// will be updated.
    ///
    /// # Arguments
    ///
    /// * `input` - The server update input containing the server ID and fields to update
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(Server)` - The updated server
    /// - `Err(CoreError::ServerNotFound)` - No server exists with the given ID
    /// - `Err(CoreError)` - If validation fails or repository operation fails
    fn update_server(
        &self,
        input: UpdateServerInput,
    ) -> impl Future<Output = Result<Server, CoreError>> + Send;

    /// Deletes a server by its unique identifier.
    ///
    /// This method validates that the server exists and that the user has permission
    /// to delete it before removing it from the repository.
    ///
    /// # Arguments
    ///
    /// * `server_id` - A reference to the unique identifier of the server to delete
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(())` - The server was successfully deleted
    /// - `Err(CoreError::ServerNotFound)` - No server exists with the given ID
    /// - `Err(CoreError)` - If repository operation fails
    fn delete_server(
        &self,
        server_id: &ServerId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_user_servers(
        &self,
        pagination: &GetPaginated,
        user_id: UserId,
    ) -> impl Future<Output = Result<(Vec<Server>, TotalPaginatedElements), CoreError>> + Send;
}

#[derive(Clone)]
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
    async fn find_by_id(&self, id: &ServerId) -> Result<Server, CoreError> {
        let servers = self.servers.lock().unwrap();

        let server = servers
            .iter()
            .find(|s| &s.id == id)
            .cloned()
            .ok_or_else(|| CoreError::ServerNotFound { id: id.clone() })?;

        Ok(server)
    }

    async fn list(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        let servers = self.servers.lock().unwrap();
        let total = servers.len() as u64;

        let offset = ((pagination.page - 1) * pagination.limit) as usize;
        let limit = pagination.limit as usize;

        let paginated_servers: Vec<Server> =
            servers.iter().skip(offset).take(limit).cloned().collect();

        Ok((paginated_servers, total))
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
            .ok_or_else(|| CoreError::ServerNotFound {
                id: input.id.clone(),
            })?;

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

    async fn list_user_servers(
        &self,
        pagination: &GetPaginated,
        _user_id: UserId,
    ) -> Result<(Vec<Server>, TotalPaginatedElements), CoreError> {
        let servers = self.servers.lock().unwrap();
        let total = servers.len() as u64;

        let offset = ((pagination.page - 1) * pagination.limit) as usize;
        let limit = pagination.limit as usize;

        let paginated_servers: Vec<Server> =
            servers.iter().skip(offset).take(limit).cloned().collect();

        Ok((paginated_servers, total))
    }
}
