use crate::{
    domain::{
        authorization::ports::AuthorizationRepository,
        channel::ports::ChannelRepository,
        channel_member::ports::ChannelMemberRepository,
        common::{GetPaginated, TotalPaginatedElements, services::Service},
        friend::{
            entities::{DeleteFriendInput, Friend, FriendRequest, UserId},
            ports::{FriendRequestService, FriendService, FriendshipRepository},
        },
        health::port::HealthRepository,
        member_role::ports::MemberRoleRepository,
        outbox::ports::OutboxRepository,
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_invitation::ports::ServerInvitationRepository,
        server_member::ports::MemberRepository,
        server_pictures::ServerPicturesRepository,
        user::port::UserRepository,
    },
    infrastructure::friend::repositories::error::FriendshipError,
};

use tracing::error;

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> FriendService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
    SC: ServerPicturesRepository,
{
    async fn get_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<Friend>, TotalPaginatedElements), FriendshipError> {
        self.friendship_repository
            .list_friends(pagination, user_id)
            .await
    }

    async fn delete_friend(&self, input: DeleteFriendInput) -> Result<(), FriendshipError> {
        self.friendship_repository.remove_friend(input).await
    }
}

impl<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC> FriendRequestService
    for Service<S, F, U, H, M, C, R, O, CM, MR, SI, A, SC>
where
    S: ServerRepository,
    F: FriendshipRepository,
    U: UserRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
    O: OutboxRepository,
    CM: ChannelMemberRepository,
    MR: MemberRoleRepository,
    SI: ServerInvitationRepository,
    A: AuthorizationRepository,
    SC: ServerPicturesRepository,
{
    async fn get_friend_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError> {
        self.friendship_repository
            .list_requests(pagination, user_id)
            .await
    }

    async fn get_friend_invitations(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError> {
        self.friendship_repository
            .list_invitations(pagination, user_id)
            .await
    }

    async fn create_friend_request(
        &self,
        user_id_requested: &UserId,
        user_pseudo_invited: &str,
    ) -> Result<FriendRequest, FriendshipError> {
        let user_id_invited = self
            .user_repository
            .get_user_by_username(&user_pseudo_invited.to_string())
            .await
            .map_err(|e| {
                error!("Error fetching user by username: {}", e);
                FriendshipError::UserNotFound
            })?
            .ok_or(FriendshipError::UserNotFound)?
            .sub;

        if user_id_requested.eq(&UserId(user_id_invited)) {
            error!(
                "User {:?} attempted to send a friend request to themselves",
                user_id_requested
            );
            return Err(FriendshipError::CannotFriendYourself);
        }

        let existing_request = self
            .friendship_repository
            .get_request(&UserId(user_id_invited), user_id_requested)
            .await?;

        if let Some(request) = existing_request {
            if request.status == 0 {
                return Err(FriendshipError::FriendshipAlreadyExists);
            } else {
                self.friendship_repository
                    .remove_request(&UserId(user_id_invited), user_id_requested)
                    .await?;
            }
        }

        let frienship = self
            .friendship_repository
            .get_friend(&UserId(user_id_invited), user_id_requested)
            .await?;
        if frienship.is_some() {
            return Err(FriendshipError::FriendshipAlreadyExists);
        }

        self.friendship_repository
            .create_request(user_id_requested, &UserId(user_id_invited))
            .await
    }

    async fn accept_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<Friend, FriendshipError> {
        self.friendship_repository
            .accept_request(user_id_requested, user_id_invited)
            .await
    }

    async fn decline_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, FriendshipError> {
        self.friendship_repository
            .decline_request(user_id_requested, user_id_invited)
            .await
    }

    async fn delete_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<(), FriendshipError> {
        self.friendship_repository
            .remove_request(user_id_requested, user_id_invited)
            .await
    }
}
