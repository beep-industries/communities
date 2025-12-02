use api::{ApiError, http::server::api_error::ErrorBody};
use axum::http::StatusCode;
use communities_core::domain::server::entities::{CreateServerRequest, ServerVisibility};
use serde_json::{Value, json};
use test_context::test_context;
use uuid::Uuid;

pub mod context;

// ============================================================================
// CREATE SERVER TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_server_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/servers")
        .json(&json!({
            "name": "Test Server",
            "owner_id": Uuid::new_v4(),
            "visibility": "Public"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_server_success(ctx: &mut context::TestContext) {
    let input = CreateServerRequest {
        name: "My Awesome Server".to_string(),
        picture_url: Some("https://example.com/picture.png".to_string()),
        banner_url: Some("https://example.com/banner.png".to_string()),
        description: Some("A test server for integration testing".to_string()),
        visibility: ServerVisibility::Public,
    };

    let res = ctx.authenticated_router.post("/servers").json(&input).await;

    res.assert_status(StatusCode::CREATED);

    let body: Value = res.json();
    assert!(body.is_object(), "response must be a JSON object");
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some("My Awesome Server")
    );
    assert!(body.get("id").is_some(), "server must have an id");
    assert!(
        body.get("created_at").is_some(),
        "server must have created_at"
    );
    assert_eq!(
        body.get("description").and_then(|v| v.as_str()),
        Some("A test server for integration testing")
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_server_empty_name_fails(ctx: &mut context::TestContext) {
    let input = CreateServerRequest {
        name: "".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let res = ctx.authenticated_router.post("/servers").json(&input).await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_server_whitespace_name_fails(ctx: &mut context::TestContext) {
    let input = CreateServerRequest {
        name: "   ".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let res = ctx.authenticated_router.post("/servers").json(&input).await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

// ============================================================================
// LIST SERVERS TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_servers_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/servers?page=1&limit=20")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_servers_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/servers?page=1&limit=20")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert!(body.is_object(), "response must be a JSON object");
    assert!(
        body.get("data").map(|v| v.is_array()).unwrap_or(false),
        "'data' field must be an array"
    );
    assert!(
        body.get("total").map(|v| v.is_number()).unwrap_or(false),
        "'total' field must be a number"
    );
    assert!(
        body.get("page").map(|v| v.is_number()).unwrap_or(false),
        "'page' field must be a number"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_servers_with_pagination(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/servers?page=2&limit=5")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(body.get("page").and_then(|v| v.as_u64()), Some(2));
}

// ============================================================================
// GET SERVER TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_server_unauthorized(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .get(&format!("/servers/{}", server_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_server_not_found(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .authenticated_router
        .get(&format!("/servers/{}", server_id))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_server_success(ctx: &mut context::TestContext) {
    // First create a server
    let input = CreateServerRequest {
        name: "Server to Get".to_string(),
        picture_url: None,
        banner_url: None,
        description: Some("Test description".to_string()),
        visibility: ServerVisibility::Public,
    };

    let create_res = ctx.authenticated_router.post("/servers").json(&input).await;

    create_res.assert_status(StatusCode::CREATED);
    let created: Value = create_res.json();
    let server_id = created.get("id").and_then(|v| v.as_str()).unwrap();

    // Now get the server
    let res = ctx
        .authenticated_router
        .get(&format!("/servers/{}", server_id))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some("Server to Get")
    );
    assert_eq!(
        body.get("description").and_then(|v| v.as_str()),
        Some("Test description")
    );
}

// ============================================================================
// UPDATE SERVER TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_server_unauthorized(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .put(&format!("/servers/{}", server_id))
        .json(&json!({
            "name": "Updated Name"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_server_not_found(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .authenticated_router
        .put(&format!("/servers/{}", server_id))
        .json(&json!({
            "name": "Updated Name"
        }))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_server_success(ctx: &mut context::TestContext) {
    // First create a server
    let input = CreateServerRequest {
        name: "Original Name".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let create_res = ctx.authenticated_router.post("/servers").json(&input).await;

    create_res.assert_status(StatusCode::CREATED);
    let created: Value = create_res.json();
    let server_id = created.get("id").and_then(|v| v.as_str()).unwrap();

    // Update the server
    let res = ctx
        .authenticated_router
        .put(&format!("/servers/{}", server_id))
        .json(&json!({
            "name": "Updated Name",
            "description": "New description"
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some("Updated Name")
    );
    assert_eq!(
        body.get("description").and_then(|v| v.as_str()),
        Some("New description")
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_server_partial_update(ctx: &mut context::TestContext) {
    // First create a server
    let input = CreateServerRequest {
        name: "Original Server".to_string(),
        picture_url: Some("old-pic.png".to_string()),
        banner_url: None,
        description: Some("Original description".to_string()),
        visibility: ServerVisibility::Public,
    };

    let create_res = ctx.authenticated_router.post("/servers").json(&input).await;

    create_res.assert_status(StatusCode::CREATED);
    let created: Value = create_res.json();
    let server_id = created.get("id").and_then(|v| v.as_str()).unwrap();

    // Partial update - only change description
    let res = ctx
        .authenticated_router
        .put(&format!("/servers/{}", server_id))
        .json(&json!({
            "description": "Updated description only"
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    // Name should remain unchanged
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some("Original Server")
    );
    // Description should be updated
    assert_eq!(
        body.get("description").and_then(|v| v.as_str()),
        Some("Updated description only")
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_server_empty_name_fails(ctx: &mut context::TestContext) {
    // First create a server
    let input = CreateServerRequest {
        name: "Valid Name".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let create_res = ctx.authenticated_router.post("/servers").json(&input).await;

    create_res.assert_status(StatusCode::CREATED);
    let created: Value = create_res.json();
    let server_id = created.get("id").and_then(|v| v.as_str()).unwrap();

    // Try to update with empty name
    let res = ctx
        .authenticated_router
        .put(&format!("/servers/{}", server_id))
        .json(&json!({
            "name": ""
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

// ============================================================================
// DELETE SERVER TESTS
// ============================================================================

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_server_unauthorized(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .unauthenticated_router
        .delete(&format!("/servers/{}", server_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_server_not_found(ctx: &mut context::TestContext) {
    let server_id = Uuid::new_v4();
    let res = ctx
        .authenticated_router
        .delete(&format!("/servers/{}", server_id))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_server_success(ctx: &mut context::TestContext) {
    // First create a server
    let input = CreateServerRequest {
        name: "Server to Delete".to_string(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: ServerVisibility::Public,
    };

    let create_res = ctx.authenticated_router.post("/servers").json(&input).await;

    create_res.assert_status(StatusCode::CREATED);
    let created: Value = create_res.json();
    let server_id = created.get("id").and_then(|v| v.as_str()).unwrap();

    // Delete the server
    let res = ctx
        .authenticated_router
        .delete(&format!("/servers/{}", server_id))
        .await;

    res.assert_status(StatusCode::OK);

    // Verify it's deleted by trying to get it
    let get_res = ctx
        .authenticated_router
        .get(&format!("/servers/{}", server_id))
        .await;

    get_res.assert_status(StatusCode::NOT_FOUND);
}
