use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::{
    domain::{
        common::{GetPaginated, TotalPaginatedElements},
        friend::entities::{DeleteFriendInput, Friend, FriendRequest, UserId},
    },
    infrastructure::friend::repositories::error::FriendshipError,
};

pub trait FriendshipRepository: Send + Sync {
    // === Friends ===
    fn list_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<Friend>, TotalPaginatedElements), FriendshipError>> + Send;

    fn get_friend(
        &self,
        user_id_1: &UserId,
        user_id_2: &UserId,
    ) -> impl Future<Output = Result<Option<Friend>, FriendshipError>> + Send;

    fn remove_friend(
        &self,
        input: DeleteFriendInput,
    ) -> impl Future<Output = Result<(), FriendshipError>> + Send;

    // === Friend Requests ===
    fn list_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError>> + Send;

    fn list_invitations(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError>> + Send;

    fn get_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<Option<FriendRequest>, FriendshipError>> + Send;

    fn create_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, FriendshipError>> + Send;

    fn accept_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<Friend, FriendshipError>> + Send;

    fn decline_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, FriendshipError>> + Send;

    fn remove_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<(), FriendshipError>> + Send;
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
    ) -> impl Future<Output = Result<(Vec<Friend>, TotalPaginatedElements), FriendshipError>> + Send;

    fn delete_friend(
        &self,
        input: DeleteFriendInput,
    ) -> impl Future<Output = Result<(), FriendshipError>> + Send;
}

pub trait FriendRequestService: Send + Sync {
    fn get_friend_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError>> + Send;

    fn get_friend_invitations(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError>> + Send;

    fn create_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &String,
    ) -> impl Future<Output = Result<FriendRequest, FriendshipError>> + Send;

    fn accept_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<Friend, FriendshipError>> + Send;

    fn decline_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, FriendshipError>> + Send;

    fn delete_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<(), FriendshipError>> + Send;
}

#[derive(Clone)]
pub struct MockFriendshipRepository {
    friends: Arc<Mutex<Vec<Friend>>>,
    friend_requests: Arc<Mutex<Vec<FriendRequest>>>,
}

impl MockFriendshipRepository {
    pub fn new() -> Self {
        Self {
            friends: Arc::new(Mutex::new(Vec::new())),
            friend_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl FriendshipRepository for MockFriendshipRepository {
    async fn list_friends(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<Friend>, TotalPaginatedElements), FriendshipError> {
        let friends = self.friends.lock().unwrap();

        let filtered_friends: Vec<Friend> = friends
            .iter()
            .filter(|friend| &friend.user_id_1 == user_id || &friend.user_id_2 == user_id)
            .cloned()
            .collect();
        let total = filtered_friends.len() as TotalPaginatedElements;
        let start = pagination.page.saturating_sub(1) * pagination.limit;

        let paginated_friends = filtered_friends
            .into_iter()
            .skip(start as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated_friends, total))
    }

    async fn get_friend(
        &self,
        user_id_1: &UserId,
        user_id_2: &UserId,
    ) -> Result<Option<Friend>, FriendshipError> {
        let friends = self.friends.lock().unwrap();

        let filtered_friends: Vec<Friend> = friends
            .iter()
            .filter(|friend| &friend.user_id_1 == user_id_1 || &friend.user_id_2 == user_id_2)
            .cloned()
            .collect();

        Ok(filtered_friends.into_iter().next())
    }

    async fn remove_friend(&self, input: DeleteFriendInput) -> Result<(), FriendshipError> {
        let mut friends = self.friends.lock().unwrap();

        let count_before = friends.len();
        friends.retain(|friend| {
            !((friend.user_id_1 == input.user_id_1 && friend.user_id_2 == input.user_id_2)
                || (friend.user_id_1 == input.user_id_2 && friend.user_id_2 == input.user_id_1))
        });

        if friends.len() == count_before {
            return Err(FriendshipError::FriendshipNotFound);
        }

        Ok(())
    }

    async fn list_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError> {
        let requests = self.friend_requests.lock().unwrap();

        let filtered_requests: Vec<FriendRequest> = requests
            .iter()
            .filter(|request| &request.user_id_requested == user_id)
            .cloned()
            .collect();
        let total = filtered_requests.len() as TotalPaginatedElements;
        let start = pagination.page.saturating_sub(1) * pagination.limit;

        let paginated_requests = filtered_requests
            .into_iter()
            .skip(start as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated_requests, total))
    }

    async fn list_invitations(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, TotalPaginatedElements), FriendshipError> {
        let requests = self.friend_requests.lock().unwrap();

        let filtered_requests: Vec<FriendRequest> = requests
            .iter()
            .filter(|request| &request.user_id_invited == user_id)
            .cloned()
            .collect();
        let total = filtered_requests.len() as TotalPaginatedElements;
        let start = pagination.page.saturating_sub(1) * pagination.limit;

        let paginated_requests = filtered_requests
            .into_iter()
            .skip(start as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated_requests, total))
    }

    async fn get_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<Option<FriendRequest>, FriendshipError> {
        let requests = self.friend_requests.lock().unwrap();

        let filtered_requests: Vec<FriendRequest> = requests
            .iter()
            .filter(|request| {
                &request.user_id_requested == user_id_requested
                    && &request.user_id_invited == user_id_invited
            })
            .cloned()
            .collect();

        Ok(filtered_requests.into_iter().next())
    }

    async fn create_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, FriendshipError> {
        let mut requests = self.friend_requests.lock().unwrap();

        // Check if a pending friend request already exists
        if requests.iter().any(|request| {
            &request.user_id_requested == user_id_requested
                && &request.user_id_invited == user_id_invited
        }) {
            return Err(FriendshipError::FriendRequestAlreadyExists);
        }

        let new_request = FriendRequest {
            user_id_requested: user_id_requested.clone(),
            user_id_invited: user_id_invited.clone(),
            created_at: Utc::now(),
            status: 0,
        };
        requests.push(new_request.clone());

        Ok(new_request)
    }

    async fn accept_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<Friend, FriendshipError> {
        let mut requests = self.friend_requests.lock().unwrap();

        if let Some(pos) = requests.iter().position(|request| {
            &request.user_id_requested == user_id_requested
                && &request.user_id_invited == user_id_invited
                && request.status == 0
        }) {
            let count_before = requests.len();
            requests.remove(pos);

            if count_before == requests.len() {
                return Err(FriendshipError::FriendRequestNotFound);
            }

            let new_friend = Friend {
                user_id_1: user_id_requested.clone(),
                user_id_2: user_id_invited.clone(),
                created_at: Utc::now(),
            };

            let mut friends = self.friends.lock().unwrap();

            if friends.iter().any(|friend| {
                (&friend.user_id_1 == user_id_requested && &friend.user_id_2 == user_id_invited)
                    || (&friend.user_id_1 == user_id_invited
                        && &friend.user_id_2 == user_id_requested)
            }) {
                return Err(FriendshipError::FriendshipAlreadyExists);
            }

            friends.push(new_friend.clone());
            Ok(new_friend)
        } else {
            Err(FriendshipError::FriendRequestNotFound)
        }
    }

    async fn decline_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, FriendshipError> {
        let mut requests = self.friend_requests.lock().unwrap();

        let request = requests.iter_mut().find(|request| {
            &request.user_id_requested == user_id_requested
                && &request.user_id_invited == user_id_invited
        });
        if let Some(request) = request {
            request.status = 1;
            Ok(request.clone())
        } else {
            Err(FriendshipError::FriendRequestNotFound)
        }
    }

    async fn remove_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<(), FriendshipError> {
        let mut requests = self.friend_requests.lock().unwrap();

        let count_before = requests.len();
        requests.retain(|request| {
            !(&request.user_id_requested == user_id_requested
                && &request.user_id_invited == user_id_invited)
        });

        if requests.len() == count_before {
            return Err(FriendshipError::FriendRequestNotFound);
        }

        Ok(())
    }
}
