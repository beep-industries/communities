use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use communities_core::domain::{
    common::GetPaginated,
    friend::entities::UserId,
    server::entities::ServerId,
    server_member::{
        entities::{ServerMember, UpdateMemberInput},
        ports::MemberService,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::http::server::{
    ApiError, AppState, Response, api_error::ErrorBody, middleware::auth::entities::UserIdentity,
    response::PaginatedResponse,
};

/// Request body for creating a new server member
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMemberRequest {
    /// User ID to add as member
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: UserId,

    /// Optional custom nickname in the server
    #[schema(example = "CoolNickname")]
    pub nickname: Option<String>,
}

/// Request body for updating a server member
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateMemberRequest {
    /// New nickname for the member
    #[schema(example = "NewNickname")]
    pub nickname: Option<String>,
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}/members",
    tag = "server_members",
    params(
        ("server_id" = String, Path, description = "Server ID"),
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of members retrieved successfully", body = PaginatedResponse<ServerMember>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden - Cannot access private server members", body = ErrorBody),
        (status = 404, description = "Server not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_members(
    Path(server_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<ServerMember>>, ApiError> {
    let server_id = ServerId::from(server_id);

    user_identity.can_view_server(server_id).await?;

    let page = pagination.page;
    let (members, total) = state.service.list_members(server_id, pagination).await?;

    let response = PaginatedResponse {
        data: members,
        total,
        page,
    };

    Ok(Response::ok(response))
}

#[utoipa::path(
    put,
    path = "/servers/{server_id}/members/{user_id}",
    tag = "server_members",
    request_body = UpdateMemberRequest,
    params(
        ("server_id" = String, Path, description = "Server ID"),
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Member updated successfully", body = ServerMember),
        (status = 400, description = "Invalid nickname", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden - Not authorized to update member", body = ErrorBody),
        (status = 404, description = "Member not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_member(
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<UpdateMemberRequest>,
) -> Result<Response<ServerMember>, ApiError> {
    let server_id = ServerId::from(server_id);
    let user_id = UserId::from(user_id);

    // Check authorization: owner or the member themselves
    user_identity
        .can_update_or_change_nickname(server_id, user_id)
        .await?;

    let input = UpdateMemberInput {
        server_id,
        user_id,
        nickname: request.nickname,
    };

    let member = state.service.update_member(input).await?;
    Ok(Response::ok(member))
}

#[utoipa::path(
    delete,
    path = "/servers/{server_id}/members/{user_id}",
    tag = "server_members",
    params(
        ("server_id" = String, Path, description = "Server ID"),
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Member removed successfully"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden - Not authorized to remove member", body = ErrorBody),
        (status = 404, description = "Member not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_member(
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<serde_json::Value>, ApiError> {
    let server_id = ServerId::from(server_id);
    let user_id = UserId::from(user_id);

    user_identity.can_manage_server(server_id).await?;

    state.service.delete_member(server_id, user_id).await?;
    Ok(Response::ok(json!({})))
}
