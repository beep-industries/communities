use uuid::Uuid;

use crate::domain::common::services::Service;
use crate::domain::common::{CoreError, GetPaginated};
use crate::domain::friend::entities::UserId;
use crate::domain::friend::ports::MockFriendshipRepository;
use crate::domain::health::port::MockHealthRepository;
use crate::domain::server::entities::{InsertServerInput, ServerVisibility};
use crate::domain::server::ports::{MockServerRepository, ServerRepository};
use crate::domain::server_member::entities::{CreateMemberInput, UpdateMemberInput};
use crate::domain::server_member::ports::{MemberRepository, MemberService, MockMemberRepository};

#[tokio::test]
#[cfg(test)]
async fn test_create_member_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let input = CreateMemberInput {
        server_id: server.id,
        user_id: UserId::from(Uuid::new_v4()),
        nickname: Some("TestUser".to_string()),
    };

    let member = service.create_member(input.clone()).await?;

    assert_eq!(member.server_id, input.server_id);
    assert_eq!(member.user_id, input.user_id);
    assert_eq!(member.nickname, Some("TestUser".to_string()));
    assert!(member.id.0 != Uuid::nil());

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_member_server_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let input = CreateMemberInput {
        server_id: Uuid::new_v4().into(),
        user_id: UserId::from(Uuid::new_v4()),
        nickname: None,
    };

    let result = service.create_member(input).await;

    assert!(matches!(result, Err(CoreError::ServerNotFound { .. })));

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_member_already_exists() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let user_id = UserId::from(Uuid::new_v4());

    // Add member directly via repository
    let first_input = CreateMemberInput {
        server_id: server.id,
        user_id,
        nickname: None,
    };
    member_mock_repo.insert(first_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    // Try to add the same member again
    let duplicate_input = CreateMemberInput {
        server_id: server.id,
        user_id,
        nickname: None,
    };

    let result = service.create_member(duplicate_input).await;

    assert!(matches!(result, Err(CoreError::MemberAlreadyExists { .. })));

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_member_invalid_nickname() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    // Test with empty nickname
    let input = CreateMemberInput {
        server_id: server.id,
        user_id: UserId::from(Uuid::new_v4()),
        nickname: Some("".to_string()),
    };

    let result = service.create_member(input).await;
    assert!(matches!(result, Err(CoreError::InvalidMemberNickname)));

    // Test with whitespace-only nickname
    let input2 = CreateMemberInput {
        server_id: server.id,
        user_id: UserId::from(Uuid::new_v4()),
        nickname: Some("   ".to_string()),
    };

    let result2 = service.create_member(input2).await;
    assert!(matches!(result2, Err(CoreError::InvalidMemberNickname)));

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_members_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    // Add multiple members
    for i in 0..3 {
        let input = CreateMemberInput {
            server_id: server.id,
            user_id: UserId::from(Uuid::new_v4()),
            nickname: Some(format!("User{}", i)),
        };
        member_mock_repo.insert(input).await?;
    }

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let pagination = GetPaginated { page: 1, limit: 10 };
    let (members, total) = service.list_members(server.id, pagination).await?;

    assert_eq!(members.len(), 3);
    assert_eq!(total, 3);

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_members_empty() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server with no members
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let pagination = GetPaginated { page: 1, limit: 10 };
    let (members, total) = service.list_members(server.id, pagination).await?;

    assert_eq!(members.len(), 0);
    assert_eq!(total, 0);

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_members_server_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let pagination = GetPaginated { page: 1, limit: 10 };
    let result = service
        .list_members(Uuid::new_v4().into(), pagination)
        .await;

    assert!(matches!(result, Err(CoreError::ServerNotFound { .. })));

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_members_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    // Add 5 members
    for i in 0..5 {
        let input = CreateMemberInput {
            server_id: server.id,
            user_id: UserId::from(Uuid::new_v4()),
            nickname: Some(format!("User{}", i)),
        };
        member_mock_repo.insert(input).await?;
    }

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    // Get page 2 with limit 2
    let pagination = GetPaginated { page: 2, limit: 2 };
    let (members, total) = service.list_members(server.id, pagination).await?;

    assert_eq!(members.len(), 2);
    assert_eq!(total, 5);

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_member_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let user_id = UserId::from(Uuid::new_v4());

    // Create member
    let create_input = CreateMemberInput {
        server_id: server.id,
        user_id,
        nickname: Some("OldNickname".to_string()),
    };
    member_mock_repo.insert(create_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    // Update member
    let update_input = UpdateMemberInput {
        server_id: server.id,
        user_id,
        nickname: Some("NewNickname".to_string()),
    };

    let updated_member = service.update_member(update_input).await?;

    assert_eq!(updated_member.nickname, Some("NewNickname".to_string()));
    assert!(updated_member.updated_at.is_some());

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_member_partial() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let user_id = UserId::from(Uuid::new_v4());

    // Create member
    let create_input = CreateMemberInput {
        server_id: server.id,
        user_id,
        nickname: Some("OriginalNickname".to_string()),
    };
    member_mock_repo.insert(create_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    // Update with None nickname (should remain unchanged)
    let update_input = UpdateMemberInput {
        server_id: server.id,
        user_id,
        nickname: None,
    };

    let updated_member = service.update_member(update_input).await?;

    // Nickname should remain unchanged
    assert_eq!(
        updated_member.nickname,
        Some("OriginalNickname".to_string())
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_member_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let update_input = UpdateMemberInput {
        server_id: Uuid::new_v4().into(),
        user_id: UserId::from(Uuid::new_v4()),
        nickname: None,
    };

    let result = service.update_member(update_input).await;

    assert!(matches!(result, Err(CoreError::MemberNotFound { .. })));

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_member_invalid_nickname() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let user_id = UserId::from(Uuid::new_v4());

    // Create member
    let create_input = CreateMemberInput {
        server_id: server.id,
        user_id,
        nickname: Some("ValidNickname".to_string()),
    };
    member_mock_repo.insert(create_input).await?;

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    // Update with empty nickname
    let update_input = UpdateMemberInput {
        server_id: server.id,
        user_id,
        nickname: Some("".to_string()),
    };

    let result = service.update_member(update_input).await;

    assert!(matches!(result, Err(CoreError::InvalidMemberNickname)));

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_delete_member_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    // Create test server
    let server_input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let server = server_mock_repo.insert(server_input).await?;

    let user_id = UserId::from(Uuid::new_v4());

    // Create member
    let create_input = CreateMemberInput {
        server_id: server.id,
        user_id,
        nickname: None,
    };
    member_mock_repo.insert(create_input).await?;

    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo.clone(),
    );

    // Delete member
    service.delete_member(server.id, user_id).await?;

    // Verify member is deleted
    let result = member_mock_repo
        .find_by_server_and_user(&server.id, &user_id)
        .await;

    assert!(matches!(result, Err(CoreError::MemberNotFound { .. })));
    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_delete_member_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let member_mock_repo = MockMemberRepository::new();

    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        member_mock_repo,
    );

    let result = service
        .delete_member(Uuid::new_v4().into(), UserId::from(Uuid::new_v4()))
        .await;

    assert!(matches!(result, Err(CoreError::MemberNotFound { .. })));

    Ok(())
}
