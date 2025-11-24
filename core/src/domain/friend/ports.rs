use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::domain::{
    common::{CoreError, GetPaginated},
    friend::entities::{
        DeleteFriendInput, Friend, FriendRequest, UserId,
    },
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
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn accept_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<Friend, CoreError>> + Send;

    fn decline_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn remove_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
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
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn accept_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<Friend, CoreError>> + Send;

    fn decline_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<FriendRequest, CoreError>> + Send;

    fn delete_friend_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

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
    ) -> Result<(Vec<Friend>, u64), CoreError> {
        let friends = self.friends
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;
        
        let filtered_friends: Vec<Friend> = friends
            .iter()
            .filter(|friend| &friend.user_id_1 == user_id || &friend.user_id_2 == user_id)
            .cloned()
            .collect();
        let total = filtered_friends.len() as u64;
        let start = pagination.page.saturating_sub(1) * pagination.limit;

        let paginated_friends = filtered_friends
            .into_iter()
            .skip(start as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated_friends, total))
    }

    async fn remove_friend(
        &self,
        input: DeleteFriendInput,
    ) -> Result<(), CoreError> {
        let mut friends = self.friends
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;

        let count_before = friends.len();
        friends.retain(|friend| {
            !( (friend.user_id_1 == input.user_id_1 && friend.user_id_2 == input.user_id_2) ||
               (friend.user_id_1 == input.user_id_2 && friend.user_id_2 == input.user_id_1) )
        });

        if friends.len() == count_before {
            return Err(CoreError::FailedToRemoveFriendship {
                user1: input.user_id_1,
                user2: input.user_id_2,
            });
        }

        Ok(())
    }

    async fn list_requests(
        &self,
        pagination: &GetPaginated,
        user_id: &UserId,
    ) -> Result<(Vec<FriendRequest>, u64), CoreError> {
        let requests = self.friend_requests
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;

        let filtered_requests: Vec<FriendRequest> = requests
            .iter()
            .filter(|request| &request.user_id_requested == user_id)
            .cloned()
            .collect();
        let total = filtered_requests.len() as u64;
        let start = pagination.page.saturating_sub(1) * pagination.limit;

        let paginated_requests = filtered_requests
            .into_iter()
            .skip(start as usize)
            .take(pagination.limit as usize)
            .collect();

        Ok((paginated_requests, total))
    }

    async fn create_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, CoreError> {
        let mut requests = self.friend_requests
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;

        // Check if a pending friend request already exists
        if requests.iter().any(|request| {
            &request.user_id_requested == user_id_requested &&
            &request.user_id_invited == user_id_invited &&
            request.status == 0
        }) {
            return Err(CoreError::FailedToCreateFriendship {
                user1: user_id_requested.clone(),
                user2: user_id_invited.clone(),
            });
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
    ) -> Result<Friend, CoreError> {
        let mut requests = self.friend_requests
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;

        if let Some(pos) = requests.iter().position(|request| {
            &request.user_id_requested == user_id_requested &&
            &request.user_id_invited == user_id_invited &&
            request.status == 0
        }) {
            requests.remove(pos);
            let new_friend = Friend {
                user_id_1: user_id_requested.clone(),
                user_id_2: user_id_invited.clone(),
                created_at: Utc::now(),
            };

            let mut friends = self.friends
                .lock()
                .map_err(|_| CoreError::MutexLockPoisoned)?;

            if friends.iter().any(|friend| {
                ( &friend.user_id_1 == user_id_requested && &friend.user_id_2 == user_id_invited ) ||
                ( &friend.user_id_1 == user_id_invited && &friend.user_id_2 == user_id_requested )
            }) {
                return Err(CoreError::FailedToCreateFriendship {
                    user1: user_id_requested.clone(),
                    user2: user_id_invited.clone(),
                });
            }

            friends.push(new_friend.clone());
            Ok(new_friend)
        } else {
            Err(CoreError::FailedToRemoveFriendship {
                user1: user_id_invited.clone(),
                user2: user_id_requested.clone(),
            })
        }
    }

    async fn decline_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<FriendRequest, CoreError> {
        let mut requests = self.friend_requests
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;

        let request = requests.iter_mut().find(|request| {
            &request.user_id_requested == user_id_requested &&
            &request.user_id_invited == user_id_invited &&
            request.status == 0
        });
        if let Some(request) = request {
            request.status = 1;
            Ok(request.clone())
        } else {
            Err(CoreError::FailedToRemoveFriendship {
                user1: user_id_invited.clone(),
                user2: user_id_requested.clone(),
            })
        }
    }

    async fn remove_request(
        &self,
        user_id_requested: &UserId,
        user_id_invited: &UserId,
    ) -> Result<(), CoreError> {
        let mut requests = self.friend_requests
            .lock()
            .map_err(|_| CoreError::MutexLockPoisoned)?;

        let count_before = requests.len();
        requests.retain(|request| {
            !( &request.user_id_requested == user_id_requested &&
               &request.user_id_invited == user_id_invited )
        });

        if requests.len() == count_before {
            return Err(CoreError::FailedToRemoveFriendship {
                user1: user_id_invited.clone(),
                user2: user_id_requested.clone(),
            });
        }

        Ok(())
    }
}