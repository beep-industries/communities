use sqlx::PgPool;

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    friend::entities::UserId,
    server::entities::ServerId,
    server_invitation::{
        entities::{
            InsertServerInvitationInput, ServerInvitation, ServerInvitationId,
            UpdateServerInvitationInput,
        },
        ports::ServerInvitationRepository,
    },
};

#[derive(Clone)]
pub struct PostgresServerInvitationRepository {
    pub(crate) pool: PgPool,
}

impl PostgresServerInvitationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ServerInvitationRepository for PostgresServerInvitationRepository {
    async fn insert(
        &self,
        _input: InsertServerInvitationInput,
    ) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn find_by_id(&self, _id: &ServerInvitationId) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn list_by_invitee(
        &self,
        _invitee_id: &Option<UserId>,
        _pagination: &GetPaginated,
    ) -> Result<(Vec<ServerInvitation>, TotalPaginatedElements), CoreError> {
        todo!()
    }

    async fn update(
        &self,
        _input: UpdateServerInvitationInput,
    ) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn delete(&self, _id: &ServerInvitationId) -> Result<(), CoreError> {
        todo!()
    }
}
