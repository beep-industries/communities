use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    role::entities::{CreateRoleInput, Role, RoleId, UpdateRoleInput, UpdateRoleRepoInput},
};

pub trait RoleRepository: Send + Sync {
    fn create(
        &self,
        create_role_input: CreateRoleInput,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn find_by_id(&self, id: &RoleId) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn list_by_server(
        &self,
        pagination: &GetPaginated,
        server_id: Uuid,
    ) -> impl Future<Output = Result<(Vec<Role>, TotalPaginatedElements), CoreError>> + Send;
    fn update(
        &self,
        update_role_input: UpdateRoleRepoInput,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn delete(&self, id: &RoleId) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait RoleService: Send + Sync {
    fn create_role(
        &self,
        create_role_input: CreateRoleInput,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn get_role(&self, role_id: &RoleId) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn list_roles_by_server(
        &self,
        pagination: &GetPaginated,
        server_id: Uuid,
    ) -> impl Future<Output = Result<(Vec<Role>, TotalPaginatedElements), CoreError>> + Send;
    fn update_role(
        &self,
        update_role_input: UpdateRoleInput,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn delete_role(&self, role_id: &RoleId) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[derive(Clone)]
pub struct MockRoleRepository {
    roles: Arc<Mutex<Vec<Role>>>,
}

impl MockRoleRepository {
    pub fn new() -> Self {
        Self {
            roles: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl RoleRepository for MockRoleRepository {
    async fn create(&self, create_role_input: CreateRoleInput) -> Result<Role, CoreError> {
        let mut roles = self.roles.lock().unwrap();

        let new_role = Role {
            id: Uuid::new_v4().into(),
            server_id: create_role_input.server_id,
            name: create_role_input.name,
            permissions: create_role_input.permissions,
            created_at: Utc::now(),
            updated_at: None,
        };

        roles.push(new_role.clone());
        Ok(new_role)
    }

    async fn find_by_id(&self, id: &RoleId) -> Result<Role, CoreError> {
        let roles = self.roles.lock().unwrap();

        roles
            .iter()
            .find(|role| &role.id == id)
            .cloned()
            .ok_or_else(|| CoreError::Error {
                msg: format!("Role with id {} not found", id),
            })
    }

    async fn list_by_server(
        &self,
        pagination: &GetPaginated,
        server_id: Uuid,
    ) -> Result<(Vec<Role>, TotalPaginatedElements), CoreError> {
        let roles = self.roles.lock().unwrap();

        let filtered_roles: Vec<Role> = roles
            .iter()
            .filter(|role| role.server_id == server_id)
            .cloned()
            .collect();

        let total = filtered_roles.len() as TotalPaginatedElements;
        let start = pagination.page.saturating_sub(1) * pagination.limit;

        let paginated_roles = filtered_roles
            .into_iter()
            .skip(start as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated_roles, total))
    }

    async fn update(&self, update_role_input: UpdateRoleRepoInput) -> Result<Role, CoreError> {
        let mut roles = self.roles.lock().unwrap();

        let role = roles
            .iter_mut()
            .find(|role| role.id == update_role_input.id)
            .ok_or_else(|| CoreError::Error {
                msg: format!("Role with id {} not found", update_role_input.id),
            })?;

        if let Some(name) = update_role_input.name {
            role.name = name;
        }

        if let Some(permissions) = update_role_input.permissions {
            role.permissions = permissions;
        }

        role.updated_at = Some(Utc::now());

        Ok(role.clone())
    }

    async fn delete(&self, id: &RoleId) -> Result<(), CoreError> {
        let mut roles = self.roles.lock().unwrap();

        let count_before = roles.len();
        roles.retain(|role| &role.id != id);

        if roles.len() == count_before {
            return Err(CoreError::Error {
                msg: format!("Role with id {} not found", id),
            });
        }

        Ok(())
    }
}
