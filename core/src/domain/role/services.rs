use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        role::{
            entities::{CreateRoleInput, Role, RoleId, UpdateRoleInput},
            ports::{RoleRepository, RoleService},
        },
        server::ports::ServerRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R> RoleService for Service<S, F, H, M, C, R>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
{
    async fn create_role(&self, create_role_input: CreateRoleInput) -> Result<Role, CoreError> {
        self.role_repository.create(create_role_input).await
    }

    async fn get_role(&self, role_id: &RoleId) -> Result<Role, CoreError> {
        self.role_repository.find_by_id(role_id).await
    }

    async fn list_roles_by_server(
        &self,
        pagination: &GetPaginated,
        server_id: uuid::Uuid,
    ) -> Result<(Vec<Role>, TotalPaginatedElements), CoreError> {
        self.role_repository
            .list_by_server(pagination, server_id)
            .await
    }

    async fn update_role(&self, update_role_input: UpdateRoleInput) -> Result<Role, CoreError> {
        self.role_repository.update(update_role_input).await
    }

    async fn delete_role(&self, server_id: &RoleId) -> Result<(), CoreError> {
        todo!()
    }
}
