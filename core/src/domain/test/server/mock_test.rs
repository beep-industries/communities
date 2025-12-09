use crate::{
    Service,
    domain::{
        common::{CoreError, GetPaginated},
        friend::{entities::UserId, ports::MockFriendshipRepository},
        health::port::MockHealthRepository,
        server::{
            entities::{InsertServerInput, ServerId, ServerVisibility, UpdateServerInput},
            ports::{MockServerRepository, ServerRepository, ServerService},
        },
        server_member::ports::MockMemberRepository,
    },
};
use uuid::Uuid;

// == Create Server Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_create_server_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: Some("https://example.com/picture.png".to_string()),
        banner_url: Some("https://example.com/banner.png".to_string()),
        description: Some("A test server".to_string()),
        visibility: ServerVisibility::Public,
    };

    let server = service
        .create_server(input.clone())
        .await
        .expect("create_server returned an error");

    assert_eq!(server.name, "Test Server", "Expected correct server name");
    assert_eq!(server.owner_id, input.owner_id, "Expected correct owner ID");
    assert_eq!(
        server.visibility,
        ServerVisibility::Public,
        "Expected public visibility"
    );
    assert_eq!(
        server.picture_url,
        Some("https://example.com/picture.png".to_string()),
        "Expected correct picture URL"
    );
    assert_eq!(
        server.banner_url,
        Some("https://example.com/banner.png".to_string()),
        "Expected correct banner URL"
    );
    assert_eq!(
        server.description,
        Some("A test server".to_string()),
        "Expected correct description"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_server_fail_empty_name() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let input = InsertServerInput {
        name: "".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let error = service
        .create_server(input)
        .await
        .expect_err("create_server should have returned an error");

    assert_eq!(
        error.to_string(),
        "Server name cannot be empty",
        "Expected invalid server name error"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_server_fail_whitespace_name() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let input = InsertServerInput {
        name: "   ".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let error = service
        .create_server(input)
        .await
        .expect_err("create_server should have returned an error");

    assert_eq!(
        error.to_string(),
        "Server name cannot be empty",
        "Expected invalid server name error"
    );

    Ok(())
}

// == Get Server Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_get_server_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a server using repository
    let input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let created_server = server_mock_repo.insert(input).await?;

    // Get the server
    let server = service
        .get_server(&created_server.id)
        .await
        .expect("get_server returned an error");

    assert_eq!(server.id, created_server.id, "Expected same server ID");
    assert_eq!(server.name, "Test Server", "Expected correct server name");

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_get_server_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let non_existent_id = ServerId::from(Uuid::new_v4());
    let error = service
        .get_server(&non_existent_id)
        .await
        .expect_err("get_server should have returned an error");

    assert!(
        error.to_string().contains("not found"),
        "Expected server not found error"
    );

    Ok(())
}

// == List Servers Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_list_servers_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert multiple servers
    for i in 1..=3 {
        let input = InsertServerInput {
            name: format!("Test Server {}", i),
            owner_id: UserId::from(Uuid::new_v4()),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: ServerVisibility::Public,
        };
        server_mock_repo.insert(input).await?;
    }

    let (servers, total) = service
        .list_servers(&GetPaginated::default())
        .await
        .expect("list_servers returned an error");

    assert_eq!(servers.len(), 3, "Expected 3 servers in the list");
    assert_eq!(total, 3, "Expected total count to be 3");

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_servers_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert 25 servers
    for i in 1..=25 {
        let input = InsertServerInput {
            name: format!("Test Server {}", i),
            owner_id: UserId::from(Uuid::new_v4()),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: ServerVisibility::Public,
        };
        server_mock_repo.insert(input).await?;
    }

    // Test page 1
    let pagination1 = GetPaginated { page: 1, limit: 10 };
    let (servers1, total1) = service
        .list_servers(&pagination1)
        .await
        .expect("list_servers page 1 returned an error");

    assert_eq!(servers1.len(), 10, "Expected 10 servers on page 1");
    assert_eq!(total1, 25, "Expected total count to be 25");

    // Test page 2
    let pagination2 = GetPaginated { page: 2, limit: 10 };
    let (servers2, total2) = service
        .list_servers(&pagination2)
        .await
        .expect("list_servers page 2 returned an error");

    assert_eq!(servers2.len(), 10, "Expected 10 servers on page 2");
    assert_eq!(total2, 25, "Expected total count to be 25");

    // Test page 3
    let pagination3 = GetPaginated { page: 3, limit: 10 };
    let (servers3, total3) = service
        .list_servers(&pagination3)
        .await
        .expect("list_servers page 3 returned an error");

    assert_eq!(servers3.len(), 5, "Expected 5 servers on page 3");
    assert_eq!(total3, 25, "Expected total count to be 25");

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_servers_empty() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let (servers, total) = service
        .list_servers(&GetPaginated::default())
        .await
        .expect("list_servers returned an error");

    assert_eq!(servers.len(), 0, "Expected empty server list");
    assert_eq!(total, 0, "Expected total count to be 0");

    Ok(())
}

// == Update Server Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_update_server_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a server
    let input = InsertServerInput {
        name: "Original Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let created_server = server_mock_repo.insert(input).await?;

    // Update the server
    let update_input = UpdateServerInput {
        id: created_server.id.clone(),
        name: Some("Updated Server".to_string()),
        picture_url: None,
        banner_url: None,
        description: Some("Updated description".to_string()),
        visibility: Some(ServerVisibility::Private),
    };

    let updated_server = service
        .update_server(update_input)
        .await
        .expect("update_server returned an error");

    assert_eq!(
        updated_server.name, "Updated Server",
        "Expected updated name"
    );
    assert_eq!(
        updated_server.description,
        Some("Updated description".to_string()),
        "Expected updated description"
    );
    assert_eq!(
        updated_server.visibility,
        ServerVisibility::Private,
        "Expected updated visibility"
    );
    assert!(
        updated_server.updated_at.is_some(),
        "Expected updated_at to be set"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_server_partial_update() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a server
    let input = InsertServerInput {
        name: "Original Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: Some("https://example.com/original.png".to_string()),
        banner_url: None,
        description: Some("Original description".to_string()),
        visibility: ServerVisibility::Public,
    };
    let created_server = server_mock_repo.insert(input).await?;

    // Update only the name
    let update_input = UpdateServerInput {
        id: created_server.id.clone(),
        name: Some("Updated Name Only".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };

    let updated_server = service
        .update_server(update_input)
        .await
        .expect("update_server returned an error");

    assert_eq!(
        updated_server.name, "Updated Name Only",
        "Expected updated name"
    );
    assert_eq!(
        updated_server.description,
        Some("Original description".to_string()),
        "Expected unchanged description"
    );
    assert_eq!(
        updated_server.picture_url,
        Some("https://example.com/original.png".to_string()),
        "Expected unchanged picture URL"
    );
    assert_eq!(
        updated_server.visibility,
        ServerVisibility::Public,
        "Expected unchanged visibility"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_server_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let update_input = UpdateServerInput {
        id: ServerId::from(Uuid::new_v4()),
        name: Some("Updated Server".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };

    let error = service
        .update_server(update_input)
        .await
        .expect_err("update_server should have returned an error");

    assert!(
        error.to_string().contains("not found"),
        "Expected server not found error"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_server_fail_empty_name() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a server
    let input = InsertServerInput {
        name: "Original Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let created_server = server_mock_repo.insert(input).await?;

    // Try to update with empty name
    let update_input = UpdateServerInput {
        id: created_server.id.clone(),
        name: Some("".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };

    let error = service
        .update_server(update_input)
        .await
        .expect_err("update_server should have returned an error");

    assert_eq!(
        error.to_string(),
        "Server name cannot be empty",
        "Expected invalid server name error"
    );

    Ok(())
}

// == Delete Server Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_delete_server_success() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo.clone(),
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a server
    let input = InsertServerInput {
        name: "Test Server".to_string(),
        owner_id: UserId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };
    let created_server = server_mock_repo.insert(input).await?;

    // Delete the server
    service
        .delete_server(&created_server.id)
        .await
        .expect("delete_server returned an error");

    // Verify server is deleted - should return error
    let deleted_server = server_mock_repo.find_by_id(&created_server.id).await;
    assert!(deleted_server.is_err(), "Expected server to be deleted");
    match deleted_server {
        Err(CoreError::ServerNotFound { .. }) => {}
        _ => panic!("Expected ServerNotFound error"),
    }

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_delete_server_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let server_mock_repo = MockServerRepository::new();
    let friend_mock_repo = MockFriendshipRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        server_mock_repo,
        friend_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let non_existent_id = ServerId::from(Uuid::new_v4());
    let error = service
        .delete_server(&non_existent_id)
        .await
        .expect_err("delete_server should have returned an error");

    assert!(
        error.to_string().contains("not found"),
        "Expected server not found error"
    );

    Ok(())
}
