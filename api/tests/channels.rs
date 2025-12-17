use api::{ApiError, http::server::api_error::ErrorBody};
use axum::http::StatusCode;
use communities_core::domain::{
    server::entities::{CreateServerRequest, ServerVisibility},
};
use serde_json::{Value, json};
use test_context::test_context;
use uuid::Uuid;

pub mod context;

// ============================================================================
// CREATE CHANNEL TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_channel_unauthorized(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&json!({
            "name": "general",
            "server_id": server_id,
            "parent_id": null,
            "channel_type": "ServerText"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_channel_success(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    server_res.assert_status(StatusCode::CREATED);
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // Now create a channel
    let channel_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;

    res.assert_status(StatusCode::CREATED);

    let body: Value = res.json();
    assert!(body.is_object(), "response must be a JSON object");
    assert_eq!(body.get("name").and_then(|v| v.as_str()), Some("general"));
    assert!(body.get("id").is_some(), "channel must have an id");
    assert_eq!(
        body.get("server_id").and_then(|v| v.as_str()),
        Some(server_id)
    );
    assert_eq!(body.get("parent_id"), Some(&Value::Null));
    assert_eq!(
        body.get("channel_type").and_then(|v| v.as_str()),
        Some("ServerText")
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_channel_with_parent(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // Create a parent folder channel
    let parent_input = json!({
        "name": "Category",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerFolder"
    });

    let parent_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&parent_input)
        .await;
    parent_res.assert_status(StatusCode::CREATED);
    let parent: Value = parent_res.json();
    let parent_id = parent.get("id").and_then(|v| v.as_str()).unwrap();

    // Create a child channel
    let child_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": parent_id,
        "channel_type": "ServerText"
    });

    let res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&child_input)
        .await;

    res.assert_status(StatusCode::CREATED);

    let body: Value = res.json();
    assert_eq!(body.get("name").and_then(|v| v.as_str()), Some("general"));
    assert_eq!(
        body.get("parent_id").and_then(|v| v.as_str()),
        Some(parent_id)
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_channel_short_name_fails(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // Try to create a channel with name too short
    let channel_input = json!({
        "name": "a",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_channel_long_name_fails(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // Try to create a channel with name too long (> 30 chars)
    let channel_input = json!({
        "name": "a".repeat(31),
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_channel_server_not_found(ctx: &mut context::TestContext) {
    let fake_server_id = Uuid::new_v4();

    let channel_input = json!({
        "name": "general",
        "server_id": fake_server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", fake_server_id))
        .json(&channel_input)
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

// ============================================================================
// LIST CHANNELS TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_channels_unauthorized(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .get(&format!("/servers/{}/channels", server_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_channels_success(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // Create multiple channels
    for name in &["general", "random", "announcements"] {
        let channel_input = json!({
            "name": name,
            "server_id": server_id,
            "parent_id": null,
            "channel_type": "ServerText"
        });

        ctx.authenticated_router
            .post(&format!("/servers/{}/channels", server_id))
            .json(&channel_input)
            .await;
    }

    // List channels
    let res = ctx
        .authenticated_router
        .get(&format!("/servers/{}/channels", server_id))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert!(body.is_array(), "response must be an array");
    let channels = body.as_array().unwrap();
    assert_eq!(channels.len(), 3, "should have 3 channels");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_channels_empty(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // List channels (should be empty)
    let res = ctx
        .authenticated_router
        .get(&format!("/servers/{}/channels", server_id))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert!(body.is_array(), "response must be an array");
    let channels = body.as_array().unwrap();
    assert_eq!(channels.len(), 0, "should have 0 channels");
}

// ============================================================================
// GET CHANNEL TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_channel_unauthorized(ctx: &mut context::TestContext) {
    let channel_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .get(&format!("/channels/{}", channel_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_channel_success(ctx: &mut context::TestContext) {
    // First create a server and channel
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    let channel_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let channel_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;
    let channel: Value = channel_res.json();
    let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap();

    // Get the channel
    let res = ctx
        .authenticated_router
        .get(&format!("/channels/{}", channel_id))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(body.get("name").and_then(|v| v.as_str()), Some("general"));
    assert_eq!(body.get("id").and_then(|v| v.as_str()), Some(channel_id));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_channel_not_found(ctx: &mut context::TestContext) {
    let fake_channel_id = Uuid::new_v4();

    let res = ctx
        .authenticated_router
        .get(&format!("/channels/{}", fake_channel_id))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

// ============================================================================
// UPDATE CHANNEL TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_channel_unauthorized(ctx: &mut context::TestContext) {
    let channel_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .put(&format!("/channels/{}", channel_id))
        .json(&json!({
            "name": "updated-name"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_channel_name_success(ctx: &mut context::TestContext) {
    // First create a server and channel
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    let channel_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let channel_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;
    let channel: Value = channel_res.json();
    let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap();

    // Update the channel
    let update_input = json!({
        "name": "updated-general",
        "parent_id": null
    });

    let res = ctx
        .authenticated_router
        .put(&format!("/channels/{}", channel_id))
        .json(&update_input)
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some("updated-general")
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_channel_parent_success(ctx: &mut context::TestContext) {
    // First create a server
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    // Create a parent folder
    let parent_input = json!({
        "name": "Category",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerFolder"
    });

    let parent_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&parent_input)
        .await;
    let parent: Value = parent_res.json();
    let parent_id = parent.get("id").and_then(|v| v.as_str()).unwrap();

    // Create a channel without parent
    let channel_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let channel_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;
    let channel: Value = channel_res.json();
    let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap();

    // Update the channel to have a parent
    let update_input = json!({
        "name": null,
        "parent_id": parent_id
    });

    let res = ctx
        .authenticated_router
        .put(&format!("/channels/{}", channel_id))
        .json(&update_input)
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(
        body.get("parent_id").and_then(|v| v.as_str()),
        Some(parent_id)
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_channel_not_found(ctx: &mut context::TestContext) {
    let fake_channel_id = Uuid::new_v4();

    let update_input = json!({
        "name": "updated-name",
        "parent_id": null
    });

    let res = ctx
        .authenticated_router
        .put(&format!("/channels/{}", fake_channel_id))
        .json(&update_input)
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_channel_empty_payload_fails(ctx: &mut context::TestContext) {
    // First create a server and channel
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    let channel_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let channel_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;
    let channel: Value = channel_res.json();
    let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap();

    // Try to update with empty payload
    let update_input = json!({
        "name": null,
        "parent_id": null
    });

    let res = ctx
        .authenticated_router
        .put(&format!("/channels/{}", channel_id))
        .json(&update_input)
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

// ============================================================================
// DELETE CHANNEL TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_channel_unauthorized(ctx: &mut context::TestContext) {
    let channel_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .delete(&format!("/channels/{}", channel_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_channel_success(ctx: &mut context::TestContext) {
    // First create a server and channel
    let server_input = CreateServerRequest {
        name: "Test Server".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&server_input)
        .await;
    let server: Value = server_res.json();
    let server_id = server.get("id").and_then(|v| v.as_str()).unwrap();

    let channel_input = json!({
        "name": "general",
        "server_id": server_id,
        "parent_id": null,
        "channel_type": "ServerText"
    });

    let channel_res = ctx
        .authenticated_router
        .post(&format!("/servers/{}/channels", server_id))
        .json(&channel_input)
        .await;
    let channel: Value = channel_res.json();
    let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap();

    // Delete the channel
    let res = ctx
        .authenticated_router
        .delete(&format!("/channels/{}", channel_id))
        .await;

    res.assert_status(StatusCode::OK);

    // Verify it's deleted
    let get_res = ctx
        .authenticated_router
        .get(&format!("/channels/{}", channel_id))
        .await;

    get_res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_channel_not_found(ctx: &mut context::TestContext) {
    let fake_channel_id = Uuid::new_v4();

    let res = ctx
        .authenticated_router
        .delete(&format!("/channels/{}", fake_channel_id))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

