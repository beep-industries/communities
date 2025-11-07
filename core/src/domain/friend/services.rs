use crate::domain::{common::{CoreError, GetPaginated, services::Service}, friend::{entities::{DeclineFriendRequestInput, DeleteFriendInput, DeleteFriendRequestInput, Friend, FriendRequest, InsertFriendInput, InsertFriendRequestInput, UpdateFriendInput, UpdateFriendRequestInput, UserId}, ports::{FriendRepository, FriendRequestRepository, FriendRequestService, FriendService}}, server::ports::ServerRepository};

impl<S, F, FR> FriendService for Service<S, F, FR>
where
    S: ServerRepository,
    F: FriendRepository,
    FR: FriendRequestRepository,
{
    async fn get_friends(
            &self,
            pagination: &GetPaginated,
            user_id: &UserId,
        ) -> Result<(Vec<Friend>, u64), CoreError> {
        self.friend_repository.find_all(pagination, user_id).await
    }

    async fn create_friend(
            &self,
            input: InsertFriendInput,
        ) -> Result<Friend, CoreError> {
        self.friend_repository.insert(input).await
    }

    async fn update_friend(
            &self,
            input: UpdateFriendInput,
        ) -> Result<Friend, CoreError> {
        self.friend_repository.update(input).await
    }

    async fn delete_friend(
            &self,
            input: DeleteFriendInput,
        ) -> Result<(), CoreError> {
        self.friend_repository.delete(input).await
    }
}

impl<S, F, FR> FriendRequestService for Service<S, F, FR>
where
    S: ServerRepository,
    F: FriendRepository,
    FR: FriendRequestRepository,
{
    async fn get_friend_requests(
            &self,
            pagination: &GetPaginated,
            user_id: &UserId,
        ) -> Result<(Vec<super::entities::FriendRequest>, u64), CoreError> {
        self.friend_request_repository.find_all(pagination, user_id).await
    }

    async fn create_friend_request(
            &self,
            input: InsertFriendRequestInput,
        ) -> Result<FriendRequest, CoreError> {
        self.friend_request_repository.insert(input).await
    }

    async fn accept_friend_request(
            &self,
            input: super::entities::AcceptFriendRequestInput,
        ) -> Result<Friend, CoreError> {
        self.friend_request_repository.delete(
            DeleteFriendRequestInput {
                user_id_requested: input.user_id_requested.clone(),
                user_id_invited: input.user_id_invited.clone(),
            }
        ).await?;

        self.friend_repository.insert(
            InsertFriendInput {
                user_id_1: input.user_id_requested,
                user_id_2: input.user_id_invited,
            }
        ).await
    }

    async fn decline_friend_request(
            &self,
            input: DeclineFriendRequestInput,
        ) -> Result<FriendRequest, CoreError> {
        self.friend_request_repository.update(
            UpdateFriendRequestInput {
                user_id_requested: input.user_id_requested,
                user_id_invited: input.user_id_invited,
                status: 1,
            }
        ).await
    }

    async fn delete_friend_request(
            &self,
            input: DeleteFriendRequestInput,
        ) -> Result<(), CoreError> {
        self.friend_request_repository.delete(input).await
    }
}