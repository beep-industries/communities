use api::{ApiError, http::server::api_error::ErrorBody};
use axum::http::StatusCode;
use serde_json::{Value, json};
use test_context::test_context;
use uuid::Uuid;

pub mod context;

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_example(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/friends").await;
    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_authenticated_get_friends_ok(ctx: &mut context::TestContext) {
    // Call an authenticated endpoint
    let res = ctx
        .authenticated_router
        .get("/friends?page=1&limit=20")
        .await;

    res.assert_status(StatusCode::OK);

    // Check response has expected fields with correct types (not values)
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
