use api::{ApiError, http::server::api_error::ErrorBody};
use axum::http::StatusCode;
use serde_json::json;
use test_context::test_context;

pub mod context;

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_example(ctx: &mut context::TestContext) {
    let res = ctx.test_router.get("/friends").await;
    res.assert_status(StatusCode::UNAUTHORIZED);
    res.assert_json(&json!(Into::<ErrorBody>::into(ApiError::Unauthorized)));
}
