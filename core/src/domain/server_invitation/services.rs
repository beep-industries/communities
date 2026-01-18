use crate::domain::channel::ports::ChannelRepository;
use crate::domain::channel_member::ports::ChannelMemberRepository;
use crate::domain::common::services::Service;
use crate::domain::common::{CoreError, GetPaginated, TotalPaginatedElements};
use crate::domain::friend::entities::UserId;
use crate::domain::friend::ports::FriendshipRepository;
use crate::domain::health::port::HealthRepository;
use crate::domain::member_role::ports::MemberRoleRepository;
use crate::domain::outbox::ports::OutboxRepository;
use crate::domain::role::ports::RoleRepository;
use crate::domain::server::entities::ServerId;
use crate::domain::server::ports::ServerRepository;
use crate::domain::server_member::ports::MemberRepository;

use super::entities::{
    InsertServerInvitationInput, ServerInvitation, ServerInvitationId, UpdateServerInvitationInput,
};
use super::ports::{ServerInvitationRepository, ServerInvitationService};

impl<S, F, H, M, C, R, O, CM, MR, SI> ServerInvitationService
    for Service<S, F, H, M, C, R, O, CM, MR, SI>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
{
    async fn create_invitation(
        &self,
        _input: InsertServerInvitationInput,
    ) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn get_invitation(
        &self,
        _invitation_id: &ServerInvitationId,
    ) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn list_invitations(
        &self,
        _pagination: &GetPaginated,
    ) -> Result<(Vec<ServerInvitation>, TotalPaginatedElements), CoreError> {
        todo!()
    }

    async fn list_server_invitations(
        &self,
        _server_id: &ServerId,
        _pagination: &GetPaginated,
    ) -> Result<(Vec<ServerInvitation>, TotalPaginatedElements), CoreError> {
        todo!()
    }

    async fn list_user_invitations(
        &self,
        _invitee_id: &Option<UserId>,
        _pagination: &GetPaginated,
    ) -> Result<(Vec<ServerInvitation>, TotalPaginatedElements), CoreError> {
        todo!()
    }

    async fn accept_invitation(
        &self,
        _invitation_id: &ServerInvitationId,
    ) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn reject_invitation(
        &self,
        _invitation_id: &ServerInvitationId,
    ) -> Result<ServerInvitation, CoreError> {
        todo!()
    }

    async fn cancel_invitation(
        &self,
        _invitation_id: &ServerInvitationId,
    ) -> Result<(), CoreError> {
        todo!()
    }
}
