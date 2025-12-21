// == Friend requests ==

#[cfg(test)]
mod tests {
    use crate::domain::{
        common::GetPaginated,
        friend::{
            entities::{DeleteFriendInput, UserId},
            ports::{FriendRequestService, FriendService, FriendshipRepository},
        },
        test::create_mock_service,
    };

    #[tokio::test]
    async fn test_get_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await?;
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174003".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await?;

        // Test the get_friend_requests method
        let friend_requests = service
            .get_friend_requests(
                &GetPaginated::default(),
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
            )
            .await
            .expect("get_friend_requests returned an error");

        assert_eq!(
            friend_requests.0.len(),
            1,
            "Expected one friend request in the list"
        );
        assert_eq!(friend_requests.1, 1, "Expected total count to be 1");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_friend_requests_success_with_pagination()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await?;
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174003".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await?;

        let pagination = GetPaginated { page: 2, limit: 10 };

        // Test the get_friend_requests method
        let friend_requests = service
            .get_friend_requests(
                &pagination,
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
            )
            .await
            .expect("get_friend_requests returned an error");

        assert_eq!(
            friend_requests.0.len(),
            0,
            "Expected no friend requests in the list"
        );
        assert_eq!(friend_requests.1, 1, "Expected total count to be 1");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_friend_invitations_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await?;
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174003".to_string()),
            )
            .await?;

        // Test the get_friend_invitations method
        let friend_invitations = service
            .get_friend_invitations(
                &GetPaginated::default(),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await
            .expect("get_friend_invitations returned an error");

        assert_eq!(
            friend_invitations.0.len(),
            1,
            "Expected one friend invitation in the list"
        );
        assert_eq!(friend_invitations.1, 1, "Expected total count to be 1");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_friend_invitations_success_with_pagination()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await?;
        service
            .friendship_repository
            .create_request(
                &UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string()),
                &UserId::from("123e4567-e89b-12d3-a456-426614174003".to_string()),
            )
            .await?;

        let pagination = GetPaginated { page: 2, limit: 10 };

        // Test the get_friend_invitations method
        let friend_invitations = service
            .get_friend_invitations(
                &pagination,
                &UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string()),
            )
            .await
            .expect("get_friend_invitations returned an error");

        assert_eq!(
            friend_invitations.0.len(),
            0,
            "Expected no friend invitations in the list"
        );
        assert_eq!(friend_invitations.1, 1, "Expected total count to be 1");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Test the create_friend_request method
        let friend_requests = service
            .create_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect("create_friend_request returned an error");

        assert_eq!(
            friend_requests.user_id_invited, user_id_invited,
            "Expected same invited user ID"
        );
        assert_eq!(
            friend_requests.user_id_requested, user_id_requested,
            "Expected same requested user ID"
        );
        assert_eq!(
            friend_requests.status, 0,
            "Expected status to be 0 (pending)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_create_friend_requests_fail_duplicate() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await
            .expect("create_request returned an error");

        // Test the create_friend_request method
        let error1 = service
            .create_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect_err("create_friend_request should have returned an error");

        assert_eq!(
            error1.to_string(),
            "Friend request already exists",
            "Expected duplicate friend request error"
        );

        // Test the create_friend_request method
        // Case: We must not be able to create a friend request (A -> B) if a (B -> A) request already exists
        let error2 = service
            .create_friend_request(&user_id_invited, &user_id_requested)
            .await
            .expect_err("create_friend_request should have returned an error");

        assert_eq!(
            error2.to_string(),
            "Friendship already exists",
            "Expected duplicate friend request error"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_accept_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await
            .expect("create_request returned an error");

        // Test the accept_friend_request method
        let friendship = service
            .accept_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect("accept_friend_request returned an error");

        assert_eq!(
            friendship.user_id_1.to_string(),
            user_id_requested.to_string(),
            "Expected same invited user ID"
        );
        assert_eq!(
            friendship.user_id_2.to_string(),
            user_id_invited.to_string(),
            "Expected same requested user ID"
        );

        // Should delete the request after accepting
        let friend_requests = service
            .friendship_repository
            .list_requests(&Default::default(), &user_id_requested)
            .await
            .expect("list_requests returned an error");

        assert_eq!(
            friend_requests.0.len(),
            0,
            "Expected no friend requests in the list after acceptance"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_accept_friend_requests_fail() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Test the accept_friend_request method
        let error = service
            .accept_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect_err("accept_friend_request should have returned an error");

        assert_eq!(
            error.to_string(),
            "Friend request not found",
            "Expected duplicate friend request error"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_decline_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await
            .expect("create_request returned an error");

        // Test the decline_friend_request method
        let friend_request = service
            .decline_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect("decline_friend_request returned an error");

        assert_eq!(
            friend_request.user_id_requested.to_string(),
            user_id_requested.to_string(),
            "Expected same requested user ID"
        );
        assert_eq!(
            friend_request.user_id_invited.to_string(),
            user_id_invited.to_string(),
            "Expected same invited user ID"
        );
        assert_eq!(
            friend_request.status, 1,
            "Expected status to be 1 (refused)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_decline_friend_requests_fail() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Test the decline_friend_request method
        let error = service
            .decline_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect_err("decline_friend_request should have returned an error");

        assert_eq!(
            error.to_string(),
            "Friend request not found",
            "Expected duplicate friend request error"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_friend_requests_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await
            .expect("create_request returned an error");

        // Test the delete_friend_request method
        service
            .delete_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect("delete_friend_request returned an error");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_friend_requests_fail() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Test the delete_friend_request method
        let error = service
            .delete_friend_request(&user_id_requested, &user_id_invited)
            .await
            .expect_err("delete_friend_request should have returned an error");

        assert_eq!(
            error.to_string(),
            "Friend request not found",
            "Expected duplicate friend request error"
        );

        Ok(())
    }

    // == Friends ==

    #[tokio::test]
    async fn test_get_friends_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await?;
        service
            .friendship_repository
            .accept_request(&user_id_requested, &user_id_invited)
            .await?;

        // Test the get_friends method
        let friends1 = service
            .get_friends(&GetPaginated::default(), &user_id_requested)
            .await
            .expect("get_friends returned an error");

        assert_eq!(friends1.0.len(), 1, "Expected one friend in the list");
        assert_eq!(friends1.1, 1, "Expected total count to be 1");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_friends_success_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await?;
        service
            .friendship_repository
            .accept_request(&user_id_requested, &user_id_invited)
            .await?;
        let pagination = GetPaginated { page: 2, limit: 10 };

        // Test the get_friends method
        let friends1 = service
            .get_friends(&pagination, &user_id_requested)
            .await
            .expect("get_friends returned an error");

        assert_eq!(friends1.0.len(), 0, "Expected no friends in the list");
        assert_eq!(friends1.1, 1, "Expected total count to be 1");

        let friends2 = service
            .get_friends(&pagination, &user_id_invited)
            .await
            .expect("get_friends returned an error");

        assert_eq!(friends2.0.len(), 0, "Expected no friends in the list");
        assert_eq!(friends2.1, 1, "Expected total count to be 1");

        Ok(())
    }

    #[cfg(test)]
    #[tokio::test]
    async fn test_delete_friend_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Add dataset
        service
            .friendship_repository
            .create_request(&user_id_requested, &user_id_invited)
            .await?;
        service
            .friendship_repository
            .accept_request(&user_id_requested, &user_id_invited)
            .await?;

        // Test the delete_friend_request method
        service
            .delete_friend(DeleteFriendInput {
                user_id_1: user_id_requested,
                user_id_2: user_id_invited,
            })
            .await
            .expect("delete_friend_request returned an error");

        Ok(())
    }

    #[cfg(test)]
    #[tokio::test]
    async fn test_delete_friend_fail() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let user_id_requested = UserId::from("123e4567-e89b-12d3-a456-426614174001".to_string());
        let user_id_invited = UserId::from("123e4567-e89b-12d3-a456-426614174002".to_string());

        // Test the delete_friend method
        let error = service
            .delete_friend(DeleteFriendInput {
                user_id_1: user_id_requested,
                user_id_2: user_id_invited,
            })
            .await
            .expect_err("delete_friend should have returned an error");

        assert_eq!(
            error.to_string(),
            "Friendship not found",
            "Expected duplicate friend request error"
        );

        Ok(())
    }
}
