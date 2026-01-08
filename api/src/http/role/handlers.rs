use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use communities_core::domain::{
    common::GetPaginated,
    role::{
        entities::{
            CreateRoleInput, CreateRoleRequest, Permissions, Role, RoleId, UpdateRoleInput,
            UpdateRoleRequest,
        },
        ports::RoleService,
    },
};
use uuid::Uuid;

use crate::{
    ApiError, AppState,
    http::server::{
        Response, middleware::auth::entities::UserIdentity, response::PaginatedResponse,
    },
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
    tag = "roles",
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

#[utoipa::path(
    get,
    path = "/servers/{server_id}",
    tag = "roles",
    params(
        ("server_id" = String, Path, description = "Server ID"),
    ),
    responses(
        (status = 200, description = "Role retrieved successfully", body = PaginatedResponse<Vec<Role>>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Server is private"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_roles_by_server(
    Path(server_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<Role>>, ApiError> {
    let (data, total) = state
        .service
        .list_roles_by_server(&pagination, server_id)
        .await
        .map_err(Into::<ApiError>::into)?;
    let paginated: PaginatedResponse<Role> = PaginatedResponse {
        data,
        total,
        page: pagination.page,
    };
    Ok(paginated.into())
}

#[utoipa::path(
    put,
    path = "/servers/{server_id}/roles/{role_id}",
    tag = "roles",
    params(
        ("server_id" = String, Path, description = "Server ID"),
        ("role_id" = String, Path, description = "Role ID")
    ),
    request_body = UpdateRoleInput,
    responses(
        (status = 200, description = "Role updated successfully", body = Role),
        (status = 400, description = "Bad request - Invalid permissions"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the server owner"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_role(
    Path((_server_id, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Response<Role>, ApiError> {
    let update_role = UpdateRoleInput {
        id: RoleId(role_id),
        name: request.name,
        permissions: request.permissions,
    };
    let role = state
        .service
        .update_role(update_role)
        .await
        .map_err(Into::<ApiError>::into)?;
    Ok(Response::ok(role))
}
