use axum::{
    Extension, Json,
    extract::{Path, State},
};
use communities_core::domain::role::{
    entities::{CreateRoleInput, CreateRoleRequest, Permissions, Role, RoleId},
    ports::RoleService,
};
use uuid::Uuid;

use crate::{
    ApiError, AppState,
    http::server::{Response, middleware::auth::entities::UserIdentity},
};

#[utoipa::path(
    post,
    path = "/servers/{server_id}/roles",
    tag = "roles",
    params(
        ("server_id" = String, Path, description = "Server ID")
    ),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "", body = Role),
        (status = 400, description = "Bad request - Invalid role"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_role(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Extension(_user_identity): Extension<UserIdentity>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<Response<Role>, ApiError> {
    let permissions = Permissions::try_from(request.permissions)
        .map_err(|e| ApiError::BadRequest { msg: e.to_string() })?;
    let create_role = CreateRoleInput {
        server_id,
        name: request.name,
        permissions,
    };
    let role = state.service.create_role(create_role).await?;
    Ok(Response::created(role))
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}/roles/{role_id}",
    tag = "servers",
    params(
        ("server_id" = String, Path, description = "Server ID"),
        ("role_id" = String, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role retrieved successfully", body = Role),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Server is private"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_role(
    Path((_server_id, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<Role>, ApiError> {
    let role = state
        .service
        .get_role(&RoleId(role_id))
        .await
        .map_err(Into::<ApiError>::into)?;
    Ok(Response::ok(role))
}
