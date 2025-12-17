use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    role::entities::{
        CreateRoleInput, CreateRoleRepoInput, Role, RoleId, UpdateRoleInput, UpdateRoleRepoInput,
    },
};

pub trait RoleRepository: Send + Sync {
    fn create(
        &self,
        create_role_input: CreateRoleRepoInput,
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
    fn delete_role(&self, server_id: &RoleId)
    -> impl Future<Output = Result<(), CoreError>> + Send;
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
