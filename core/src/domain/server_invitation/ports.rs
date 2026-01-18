use std::sync::{Arc, Mutex};

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    friend::entities::UserId,
    server_invitation::entities::{
        AcceptInvitationInput, InsertServerInvitationInput, ServerInvitation, ServerInvitationId,
        UpdateServerInvitationInput,
    },
};

pub trait ServerInvitationRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertServerInvitationInput,
    ) -> impl Future<Output = Result<ServerInvitation, CoreError>> + Send;

    fn find_by_id(
        &self,
        id: &ServerInvitationId,
    ) -> impl Future<Output = Result<ServerInvitation, CoreError>> + Send;

    fn list_by_invitee(
        &self,
        invitee_id: &Option<UserId>,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<ServerInvitation>, TotalPaginatedElements), CoreError>> + Send;

    fn update(
        &self,
        input: UpdateServerInvitationInput,
    ) -> impl Future<Output = Result<ServerInvitation, CoreError>> + Send;

    fn delete(&self, id: &ServerInvitationId)
    -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait ServerInvitationService: Send + Sync {
    fn create_invitation(
        &self,
        input: InsertServerInvitationInput,
    ) -> impl Future<Output = Result<ServerInvitation, CoreError>> + Send;

    fn get_invitation(
        &self,
        invitation_id: &ServerInvitationId,
    ) -> impl Future<Output = Result<ServerInvitation, CoreError>> + Send;

    fn accept_invitation(
        &self,
        accept_input: &AcceptInvitationInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[derive(Clone)]
pub struct MockServerInvitationRepository {
    invitations: Arc<Mutex<Vec<ServerInvitation>>>,
}

impl MockServerInvitationRepository {
    pub fn new() -> Self {
        Self {
            invitations: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ServerInvitationRepository for MockServerInvitationRepository {
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
