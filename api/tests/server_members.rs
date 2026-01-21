use api::{ApiError, http::server::api_error::ErrorBody};
use axum::http::StatusCode;
use communities_core::domain::server::{
    entities::{InsertServerInput, ServerVisibility},
    ports::ServerService,
};
use serde_json::{Value, json};
use test_context::test_context;

mod context;
mod helpers;

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_members_unauthenticated(ctx: &mut context::TestContext) {
    let server_id = "550e8400-e29b-41d4-a716-446655440001";
    let res = ctx
        .unauthenticated_router
        .get(&format!("/servers/{}/members?page=1&limit=20", server_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_members_ok(ctx: &mut context::TestContext) {
    // First create a server
    let create_server_res = ctx
        .app
        .state
        .service
        .create_server(InsertServerInput {
            name: "Test Server".to_string(),
            owner_id: ctx.authenticated_user_id.into(),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: ServerVisibility::Public,
        })
        .await
        .unwrap();

    let server_id = create_server_res.id.to_string();

    // Add a member using the service directly
    use communities_core::domain::server_member::{
        entities::CreateMemberInput,
        MemberService,
    };
    use uuid::Uuid;
    
    ctx.app
        .state
        .service
        .create_member(CreateMemberInput {
            server_id: create_server_res.id,
            user_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
                .unwrap()
                .into(),
            nickname: Some("TestNickname".to_string()),
        })
        .await
        .unwrap();

    // List members
    let res = ctx
        .authenticated_router
        .get(&format!("/servers/{}/members?page=1&limit=20", server_id))
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
async fn test_list_members_pagination(ctx: &mut context::TestContext) {
    // First create a server
    let create_server_res = ctx
        .app
        .state
        .service
        .create_server(InsertServerInput {
            name: "Test Server".to_string(),
            owner_id: ctx.authenticated_user_id.into(),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: ServerVisibility::Public,
        })
        .await
        .unwrap();
    // List members with custom pagination
    let res = ctx
        .authenticated_router
        .get(&format!(
            "/servers/{}/members?page=2&limit=5",
            create_server_res.id
        ))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(
        body.get("page").and_then(|v| v.as_u64()),
        Some(2),
        "'page' must be 2"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_member_unauthenticated(ctx: &mut context::TestContext) {
    let server_id = "550e8400-e29b-41d4-a716-446655440001";
    let user_id = "550e8400-e29b-41d4-a716-446655440000";
    let res = ctx
        .unauthenticated_router
        .put(&format!("/servers/{}/members/{}", server_id, user_id))
        .json(&json!({
            "nickname": "NewNickname"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}





#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_members_private_server_when_not_member(ctx: &mut context::TestContext) {
    // Create a private server with the authenticated user
    let create_server_res = ctx
        .authenticated_router
        .post("/servers")
        .json(&json!({
            "name": "Private Test Server",
            "visibility": "Private"
        }))
        .await;

    create_server_res.assert_status(StatusCode::CREATED);
    let server: Value = create_server_res.json();
    let server_id = server["id"].as_str().unwrap();

    // Try to list members with a different user (not a member)
    let different_user_router = ctx.create_authenticated_router_with_different_user().await;
    let res = different_user_router
        .get(&format!("/servers/{}/members?page=1&limit=20", server_id))
        .await;

    // Should be forbidden since the user is not a member of the private server
    res.assert_status(StatusCode::FORBIDDEN);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_list_members_private_server_when_member(ctx: &mut context::TestContext) {
    // Create a private server (authenticated user becomes owner and member)
    let create_server_res = ctx
        .app
        .state
        .service
        .create_server(InsertServerInput {
            name: "Private Test Server".to_string(),
            owner_id: ctx.authenticated_user_id.into(),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: ServerVisibility::Private,
        })
        .await
        .unwrap();

    // List members as the owner (who is also a member)
    let res = ctx
        .authenticated_router
        .get(&format!(
            "/servers/{}/members?page=1&limit=20",
            create_server_res.id
        ))
        .await;

    // Should succeed since the authenticated user is a member (owner)
    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert!(body.is_object(), "response must be a JSON object");
    assert!(
        body.get("data").map(|v| v.is_array()).unwrap_or(false),
        "'data' field must be an array"
    );
    // Should have at least 1 member (the owner)
    let data = body.get("data").and_then(|v| v.as_array()).unwrap();
    assert!(data.len() >= 1, "Should have at least the owner as member");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_member_unauthenticated(ctx: &mut context::TestContext) {
    let server_id = "550e8400-e29b-41d4-a716-446655440001";
    let user_id = "550e8400-e29b-41d4-a716-446655440000";
    let res = ctx
        .unauthenticated_router
        .delete(&format!("/servers/{}/members/{}", server_id, user_id))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}


