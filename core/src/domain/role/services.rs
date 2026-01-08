use crate::{
    Service,
    domain::{
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        friend::ports::FriendshipRepository,
        health::port::HealthRepository,
        outbox::ports::OutboxRepository,
        role::{
            entities::{
                CreateRoleInput, Role, RoleError, RoleId, UpdateRoleInput, UpdateRoleRepoInput,
            },
            ports::{RoleRepository, RoleService},
        },
        server::ports::ServerRepository,
        server_member::MemberRepository,
    },
};

impl<S, F, H, M, C, R, O, CM> RoleService for Service<S, F, H, M, C, R, O, CM>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
{
    async fn create_role(&self, create_role_input: CreateRoleInput) -> Result<Role, CoreError> {
        let repo_input = CreateRoleInput::try_from(create_role_input).map_err(|e| {
            Into::<CoreError>::into(RoleError::BadRolePayload { msg: e.to_string() })
        })?;
        self.role_repository.create(repo_input).await
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
        let repo_input = UpdateRoleRepoInput::try_from(update_role_input).map_err(|e| {
            Into::<CoreError>::into(RoleError::BadRolePayload { msg: e.to_string() })
        })?;
        self.role_repository.update(repo_input).await
    }

    async fn delete_role(&self, role_id: &RoleId) -> Result<(), CoreError> {
        self.role_repository.delete(role_id).await
    }
}
