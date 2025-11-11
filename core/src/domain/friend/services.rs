use crate::domain::{common::{CoreError, GetPaginated, services::Service}, friend::{entities::{AcceptFriendRequestInput, CreateFriendRequestInput, DeclineFriendRequestInput, DeleteFriendInput, DeleteFriendRequestInput, Friend, FriendRequest, UserId}, ports::{FriendRequestService, FriendService, FriendshipRepository}}, server::ports::ServerRepository};

impl<S, F> FriendService for Service<S, F>
where
    S: ServerRepository,
    F: FriendshipRepository
{
    async fn get_friends(
            &self,
            pagination: &GetPaginated,
            user_id: &UserId,
        ) -> Result<(Vec<Friend>, u64), CoreError> {
        self.friendship_repository.list_friends(pagination, user_id).await
    }

    async fn delete_friend(
            &self,
            input: DeleteFriendInput,
        ) -> Result<(), CoreError> {
        self.friendship_repository.remove_friend(input).await
    }
}

impl<S, F> FriendRequestService for Service<S, F>
where
    S: ServerRepository,
    F: FriendshipRepository,
{
    async fn get_friend_requests(
            &self,
            pagination: &GetPaginated,
            user_id: &UserId,
        ) -> Result<(Vec<FriendRequest>, u64), CoreError> {
        self.friendship_repository.list_requests(pagination, user_id).await
    }

    async fn create_friend_request(
            &self,
            input: CreateFriendRequestInput,
        ) -> Result<FriendRequest, CoreError> {
        self.friendship_repository.create_request(input).await
    }

    async fn accept_friend_request(
            &self,
            input: AcceptFriendRequestInput,
        ) -> Result<Friend, CoreError> {
        self.friendship_repository.accept_request(input).await
    }

    async fn decline_friend_request(
            &self,
            input: DeclineFriendRequestInput,
        ) -> Result<FriendRequest, CoreError> {
        self.friendship_repository.decline_request(input).await
    }

    async fn delete_friend_request(
            &self,
            input: DeleteFriendRequestInput,
        ) -> Result<(), CoreError> {
        self.friendship_repository.remove_request(input).await
    }
}