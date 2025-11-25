use crate::{
    domain::{
        common::{GetPaginated, services::Service},
        friend::{
            entities::{DeleteFriendInput, Friend, FriendRequest, UserId},
            ports::{FriendRequestService, FriendService, FriendshipRepository},
        },
        health::port::HealthRepository,
        server::ports::ServerRepository,
    },
    infrastructure::friend::repositories::error::FriendshipError,
};

impl<S, F, H> FriendService for Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    async fn get_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<Friend>, u64), FriendshipError> {
        self.friendship_repository
            .list_friends(pagination, user_id)
            .await
    }

    async fn delete_friend(&self, input: DeleteFriendInput) -> Result<(), FriendshipError> {
        self.friendship_repository.remove_friend(input).await
    }
}

impl<S, F, H> FriendRequestService for Service<S, F, H>
where
    S: ServerRepository,
    F: FriendshipRepository,
    H: HealthRepository,
{
    async fn get_friend_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, u64), FriendshipError> {
        self.friendship_repository
            .list_requests(pagination, user_id)
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
            return Err(FriendshipError::FriendshipAlreadyExists {
                user1: user_id_requested.clone(),
                user2: user_id_invited.clone(),
            });
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
