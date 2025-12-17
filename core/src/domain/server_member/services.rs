use crate::domain::channel::ports::ChannelRepository;
use crate::domain::common::services::Service;
use crate::domain::common::{CoreError, GetPaginated, TotalPaginatedElements};
use crate::domain::friend::entities::UserId;
use crate::domain::friend::ports::FriendshipRepository;
use crate::domain::health::port::HealthRepository;
use crate::domain::role::ports::RoleRepository;
use crate::domain::server::entities::ServerId;
use crate::domain::server::ports::ServerRepository;

use super::entities::{CreateMemberInput, ServerMember, UpdateMemberInput};
use super::ports::{MemberRepository, MemberService};

impl<S, F, H, M, C, R> MemberService for Service<S, F, H, M, C, R>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository
{
    async fn create_member(&self, input: CreateMemberInput) -> Result<ServerMember, CoreError> {
        // Validate server exists
        let _server = self.server_repository.find_by_id(&input.server_id).await?;

        // Check if member already exists
        if let Ok(_existing) = self
            .member_repository
            .find_by_server_and_user(&input.server_id, &input.user_id)
            .await
        {
            return Err(CoreError::MemberAlreadyExists {
                server_id: input.server_id,
                user_id: input.user_id,
            });
        }

        // Validate nickname
        if let Some(ref nickname) = input.nickname {
            if nickname.trim().is_empty() {
                return Err(CoreError::InvalidMemberNickname);
            }
        }

        // Create member
        let member = self.member_repository.insert(input).await?;
        Ok(member)
    }

    async fn list_members(
        &self,
        server_id: ServerId,
        pagination: GetPaginated,
    ) -> Result<(Vec<ServerMember>, TotalPaginatedElements), CoreError> {
        // Validate server exists
        let _server = self.server_repository.find_by_id(&server_id).await?;

        // List members
        let (members, total) = self
            .member_repository
            .list_by_server(&server_id, &pagination)
            .await?;

        Ok((members, total))
    }

    async fn update_member(&self, input: UpdateMemberInput) -> Result<ServerMember, CoreError> {
        // Check if member exists

        let _ = self
            .member_repository
            .find_by_server_and_user(&input.server_id, &input.user_id)
            .await?;

        // Validate nickname if provided
        if let Some(ref nickname) = input.nickname {
            if nickname.trim().is_empty() {
                return Err(CoreError::InvalidMemberNickname);
            }
        }

        // Update member
        let member = self.member_repository.update(input).await?;
        Ok(member)
    }

    async fn delete_member(&self, server_id: ServerId, user_id: UserId) -> Result<(), CoreError> {
        // Check if member exists
        let _ = self
            .member_repository
            .find_by_server_and_user(&server_id, &user_id)
            .await?;
        // Delete member
        self.member_repository.delete(&server_id, &user_id).await?;
        Ok(())
    }

    async fn get_member(
        &self,
        server_id: ServerId,
        user_id: UserId,
    ) -> Result<ServerMember, CoreError> {
        self.member_repository
            .find_by_server_and_user(&server_id, &user_id)
            .await
    }
}
