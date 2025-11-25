// == Friend requests ==

#[cfg(test)]
#[tokio::test]
async fn test_get_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{common::GetPaginated, friend::{entities::UserId, ports::{FriendRequestService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    // Add dataset
    friend_mock_repo.create_request(&UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()), &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string())).await?;
    
    // Test the get_friend_requests method
    let friend_requests = service.get_friend_requests(&GetPaginated::default(), &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()))
        .await
        .expect("get_friend_requests returned an error");

    assert_eq!(friend_requests.0.len(), 1, "Expected one friend request in the list");
    assert_eq!(friend_requests.1, 1, "Expected total count to be 1");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_get_friend_requests_success_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{common::GetPaginated, friend::{entities::UserId, ports::{FriendRequestService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);
    
    // Add dataset
    friend_mock_repo.create_request(&UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()), &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string())).await?;
    let pagination = GetPaginated { page: 2, limit: 10 };

    // Test the get_friend_requests method
    let friend_requests = service.get_friend_requests(&pagination, &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()))
        .await
        .expect("get_friend_requests returned an error");

    assert_eq!(friend_requests.0.len(), 0, "Expected no friend requests in the list");
    assert_eq!(friend_requests.1, 1, "Expected total count to be 1");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_create_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Test the create_friend_request method
    let friend_requests = service.create_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect("create_friend_request returned an error");

    assert_eq!(friend_requests.user_id_invited, user_id_invited, "Expected same invited user ID");
    assert_eq!(friend_requests.user_id_requested, user_id_requested, "Expected same requested user ID");
    assert_eq!(friend_requests.status, 0, "Expected status to be 0 (pending)");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_create_friend_requests_fail_duplicate() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited)
        .await
        .expect("create_request returned an error");

    // Test the create_friend_request method
    let error1 = service.create_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect_err("create_friend_request should have returned an error");

    assert_eq!(error1.to_string(), format!("Friend request already exists between {} and {}", user_id_requested, user_id_invited), "Expected duplicate friend request error");

    // Test the create_friend_request method
    // Case: We must not be able to create a friend request (A -> B) if a (B -> A) request already exists
    let error2 = service.create_friend_request(&user_id_invited, &user_id_requested)
        .await
        .expect_err("create_friend_request should have returned an error");

    assert_eq!(error2.to_string(), format!("Friendship already exists between {} and {}", user_id_invited, user_id_requested), "Expected duplicate friend request error");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_accept_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited)
        .await
        .expect("create_request returned an error");

    // Test the accept_friend_request method
    let friendship = service.accept_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect("accept_friend_request returned an error");

    assert_eq!(friendship.user_id_1.to_string(), user_id_requested.to_string(), "Expected same invited user ID");
    assert_eq!(friendship.user_id_2.to_string(), user_id_invited.to_string(), "Expected same requested user ID");
    
    // Should delete the request after accepting
    let friend_requests = friend_mock_repo.list_requests(&Default::default(), &user_id_requested)
        .await
        .expect("list_requests returned an error");

    assert_eq!(friend_requests.0.len(), 0, "Expected no friend requests in the list after acceptance");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_accept_friend_requests_fail() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Test the accept_friend_request method
    let error = service.accept_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect_err("accept_friend_request should have returned an error");

    assert_eq!(error.to_string(), format!("Friend request not found between {} and {}", user_id_invited, user_id_requested), "Expected duplicate friend request error");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_decline_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited)
        .await
        .expect("create_request returned an error");

    // Test the decline_friend_request method
    let friend_request = service.decline_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect("decline_friend_request returned an error");

    assert_eq!(friend_request.user_id_requested.to_string(), user_id_requested.to_string(), "Expected same requested user ID");
    assert_eq!(friend_request.user_id_invited.to_string(), user_id_invited.to_string(), "Expected same invited user ID");
    assert_eq!(friend_request.status, 1, "Expected status to be 1 (refused)");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_decline_friend_requests_fail() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Test the decline_friend_request method
    let error = service.decline_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect_err("decline_friend_request should have returned an error");

        assert_eq!(error.to_string(), format!("Friend request not found between {} and {}", user_id_invited, user_id_requested), "Expected duplicate friend request error");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_delete_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited)
        .await
        .expect("create_request returned an error");

    // Test the delete_friend_request method
    service.delete_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect("delete_friend_request returned an error");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_delete_friend_requests_fail() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::UserId, ports::{FriendRequestService, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Test the delete_friend_request method
    let error = service.delete_friend_request(&user_id_requested, &user_id_invited)
        .await
        .expect_err("delete_friend_request should have returned an error");

        assert_eq!(error.to_string(), format!("Friend request not found between {} and {}", user_id_invited, user_id_requested), "Expected duplicate friend request error");

    Ok(())
}

// == Friends ==

#[cfg(test)]
#[tokio::test]
async fn test_get_friends_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{common::GetPaginated, friend::{entities::UserId, ports::{FriendService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited).await?;
    friend_mock_repo.accept_request(&user_id_requested, &user_id_invited).await?;
    
    // Test the get_friends method
    let friends1 = service.get_friends(&GetPaginated::default(), &user_id_requested)
        .await
        .expect("get_friends returned an error");

    assert_eq!(friends1.0.len(), 1, "Expected one friend in the list");
    assert_eq!(friends1.1, 1, "Expected total count to be 1");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_get_friends_success_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{common::GetPaginated, friend::{entities::UserId, ports::{FriendService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());
    
    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited).await?;
    friend_mock_repo.accept_request(&user_id_requested, &user_id_invited).await?;
    let pagination = GetPaginated { page: 2, limit: 10 };

    // Test the get_friends method
    let friends1 = service.get_friends(&pagination, &user_id_requested)
        .await
        .expect("get_friends returned an error");

    assert_eq!(friends1.0.len(), 0, "Expected no friends in the list");
    assert_eq!(friends1.1, 1, "Expected total count to be 1");

    let friends2 = service.get_friends(&pagination, &user_id_invited)
        .await
        .expect("get_friends returned an error");

    assert_eq!(friends2.0.len(), 0, "Expected no friends in the list");
    assert_eq!(friends2.1, 1, "Expected total count to be 1");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_delete_friend_success() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::{DeleteFriendInput, UserId}, ports::{FriendService, FriendshipRepository, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Add dataset
    friend_mock_repo.create_request(&user_id_requested, &user_id_invited).await?;
    friend_mock_repo.accept_request(&user_id_requested, &user_id_invited).await?;

    // Test the delete_friend_request method
    service.delete_friend(DeleteFriendInput { user_id_1: user_id_requested, user_id_2: user_id_invited })
        .await
        .expect("delete_friend_request returned an error");

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_delete_friend_fail() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Service, domain::{friend::{entities::{DeleteFriendInput, UserId}, ports::{FriendService, MockFriendshipRepository}}, health::port::MockHealthRepository, server::ports::MockServerRepository}};

    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(server_mock_repo, friend_mock_repo.clone(), health_mock_repo);

    let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
    let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

    // Test the delete_friend method
    let error = service.delete_friend(DeleteFriendInput { user_id_1: user_id_requested, user_id_2: user_id_invited })
        .await
        .expect_err("delete_friend should have returned an error");

        assert_eq!(error.to_string(), format!("Friendship not found between {} and {}", user_id_requested, user_id_invited), "Expected duplicate friend request error");

    Ok(())
}
