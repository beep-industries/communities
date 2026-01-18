use crate::domain::channel::ports::ChannelRepository;
use crate::domain::channel_member::ports::ChannelMemberRepository;
use crate::domain::common::CoreError;
use crate::domain::common::services::Service;
use crate::domain::friend::ports::FriendshipRepository;
use crate::domain::health::port::HealthRepository;
use crate::domain::member_role::ports::MemberRoleRepository;
use crate::domain::outbox::ports::OutboxRepository;
use crate::domain::role::ports::RoleRepository;
use crate::domain::server::ports::ServerRepository;
use crate::domain::server_invitation::entities::{AcceptInvitationInput, ServerInvitationStatus};
use crate::domain::server_member::CreateMemberInput;
use crate::domain::server_member::ports::MemberRepository;

use super::entities::{InsertServerInvitationInput, ServerInvitation, ServerInvitationId};
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
        input: InsertServerInvitationInput,
    ) -> Result<ServerInvitation, CoreError> {
        let invitation = self.server_invitation_repository.insert(input).await?;
        Ok(invitation)
    }

    async fn get_invitation(
        &self,
        invitation_id: &ServerInvitationId,
    ) -> Result<ServerInvitation, CoreError> {
        let invitation = self
            .server_invitation_repository
            .find_by_id(invitation_id)
            .await?;
        Ok(invitation)
    }

    async fn accept_invitation(
        &self,
        accept_input: &AcceptInvitationInput,
    ) -> Result<(), CoreError> {
        let invitation: ServerInvitation = self
            .server_invitation_repository
            .find_by_id(&accept_input.invitation_id)
            .await?;

        // Check if invitation is still pending
        match invitation.status {
            ServerInvitationStatus::Pending => {}
            _ => return Err(CoreError::Forbidden),
        }

        // Check if invitation is expired
        if invitation.is_expired() {
            return Err(CoreError::Forbidden);
        }

        // Check invitation type and validate user
        match invitation.invitee_id {
            // Personal invitation - only specific user can accept
            Some(invitee_id) if invitee_id == accept_input.user_id => {
                // Check if user is not already a member
                if self.member_repository
                    .find_by_server_and_user(&invitation.server_id, &invitee_id)
                    .await
                    .is_err()
                {
                    // User not a member yet, add them
                    self.member_repository
                        .insert(CreateMemberInput {
                            server_id: invitation.server_id,
                            user_id: invitee_id,
                            nickname: None,
                        })
                        .await?;
                }

                // Always delete the personal invitation after acceptance
                self.server_invitation_repository
                    .delete(&invitation.id)
                    .await?;
            }
            // General invitation - anyone can use it (invitee_id is None)
            None => {
                // Check if user is not already a member
                if self.member_repository
                    .find_by_server_and_user(&invitation.server_id, &accept_input.user_id)
                    .await
                    .is_err()
                {
                    // User not a member yet, add them
                    self.member_repository
                        .insert(CreateMemberInput {
                            server_id: invitation.server_id,
                            user_id: accept_input.user_id,
                            nickname: None,
                        })
                        .await?;
                }

                // General invitations are NOT deleted, they can be reused
            }
            // Personal invitation but wrong user trying to accept
            Some(_) => {
                return Err(CoreError::Forbidden);
            }
        }

        Ok(())
    }
}
