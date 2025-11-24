
#[cfg(test)]
#[tokio::test]
async fn test_get_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;

    use crate::domain::{common::GetPaginated, friend::{entities::UserId, ports::{FriendshipRepository, MockFriendshipRepository}}};
    let mock_repo = Arc::new(MockFriendshipRepository::new());
    
    // Add dataset
    mock_repo.create_request(&UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()), &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string())).await?;
    
    // Test the get_friend_requests method
    let friend_requests = mock_repo.list_requests(&GetPaginated::default(), &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()))
        .await
        .expect("list_requests returned an error");

    assert_eq!(friend_requests.0.len(), 1, "Expected one friend request in the list");
    assert_eq!(friend_requests.1, 1, "Expected total count to be 1");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_get_friend_requests_success_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;

    use crate::domain::{common::GetPaginated, friend::{entities::UserId, ports::{FriendshipRepository, MockFriendshipRepository}}};
    let mock_repo = Arc::new(MockFriendshipRepository::new());
    
    // Add dataset
    mock_repo.create_request(&UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()), &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string())).await?;
    let pagination = GetPaginated { page: 2, limit: 10 };

    // Test the get_friend_requests method
    let friend_requests = mock_repo.list_requests(&pagination, &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()))
        .await
        .expect("list_requests returned an error");

    assert_eq!(friend_requests.0.len(), 0, "Expected no friend requests in the list");
    assert_eq!(friend_requests.1, 1, "Expected total count to be 1");

    Ok(())
}