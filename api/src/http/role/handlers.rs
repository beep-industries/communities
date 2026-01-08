use axum::{
    Extension, Json,
    extract::{Path, State},
};
use communities_core::domain::role::{
    entities::{CreateRoleInput, CreateRoleRequest, Permissions, Role},
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
