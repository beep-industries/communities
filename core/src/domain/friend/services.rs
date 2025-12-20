use crate::{
    domain::{
        channel::ports::ChannelRepository,
        common::{GetPaginated, TotalPaginatedElements, services::Service},
        friend::{
            entities::{DeleteFriendInput, Friend, FriendRequest, UserId},
            ports::{FriendRequestService, FriendService, FriendshipRepository},
        },
        health::port::HealthRepository,
        role::ports::RoleRepository,
        server::ports::ServerRepository,
        server_member::ports::MemberRepository,
    },
    infrastructure::friend::repositories::error::FriendshipError,
};

impl<S, F, H, M, C, R> FriendService for Service<S, F, H, M, C, R>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
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

impl<S, F, H, M, C, R> FriendRequestService for Service<S, F, H, M, C, R>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
    M: MemberRepository,
    C: ChannelRepository,
    R: RoleRepository,
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
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, FriendshipError> {
        let existing_request = self
            .friendship_repository
            .get_request(user_id_invited, user_id_requested)
            .await?;

        if existing_request.is_some() {
            return Err(FriendshipError::FriendshipAlreadyExists);
        }

        self.friendship_repository
            .create_request(user_id_requested, user_id_invited)
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
