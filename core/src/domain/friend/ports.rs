use crate::domain::{
    common::{CoreError, GetPaginated}, friend::entities::{AcceptFriendRequestInput, CreateFriendRequestInput, DeclineFriendRequestInput, DeleteFriendInput, DeleteFriendRequestInput, Friend, FriendRequest, UserId}
};

pub trait FriendshipRepository: Send + Sync {
    // === Friends ===
    fn list_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<Friend>, u64), CoreError>> + Send;

    fn remove_friend(
        &self,
        input: DeleteFriendInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    // === Friend Requests ===
    fn list_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<FriendRequest>, u64), CoreError>> + Send;

    fn create_request(
        &self,
        input: CreateFriendRequestInput,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn accept_request(
        &self,
        input: AcceptFriendRequestInput,
    ) -> impl Future<Output = Result<Friend, CoreError>> + Send;

    fn decline_request(
        &self,
        input: DeclineFriendRequestInput,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn remove_request(
        &self,
        input: DeleteFriendRequestInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// A service for managing server operations in the application.
///
/// This trait defines the core business logic operations that can be performed on servers.
/// It follows the ports and adapters pattern, where this trait acts as a port that defines
/// the interface for server-related operations. Implementations of this trait will provide
/// the actual business logic while maintaining separation of concerns.
///
/// The trait requires `Send + Sync` to ensure thread safety in async contexts, making it
/// suitable for use in web servers and other concurrent applications
///
/// # Thread Safety
///
/// All implementations must be thread-safe (`Send + Sync`) to support concurrent access
/// in multi-threaded environments.
pub trait FriendService: Send + Sync {
    fn get_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<Friend>, u64), CoreError>> + Send;

    fn delete_friend(
        &self,
        input: DeleteFriendInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait FriendRequestService: Send + Sync {
    fn get_friend_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<FriendRequest>, u64), CoreError>> + Send;

    fn create_friend_request(
        &self,
        input: CreateFriendRequestInput,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn accept_friend_request(
        &self,
        input: AcceptFriendRequestInput,
    ) -> impl Future<Output = Result<Friend, CoreError>> + Send;

    fn decline_friend_request(
        &self,
        input: DeclineFriendRequestInput,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn delete_friend_request(
        &self,
        input: DeleteFriendRequestInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}