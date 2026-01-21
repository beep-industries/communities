use api::{ApiError, http::server::api_error::ErrorBody};
use axum::http::StatusCode;
use communities_core::domain::server::entities::{CreateServerRequest, ServerVisibility};
use serde_json::{Value, json};
use test_context::test_context;
use uuid::Uuid;

pub mod context;
pub mod helpers;

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
async fn test_delete_channel_not_found(ctx: &mut context::TestContext) {
    let fake_channel_id = Uuid::new_v4();

    let res = ctx
        .authenticated_router
        .delete(&format!("/channels/{}", fake_channel_id))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}
