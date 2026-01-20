use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use communities_core::domain::{
    common::GetPaginated,
    member_role::{entities::MemberRole, ports::MemberRoleService},
    role::{
        entities::{
            CreateRoleInput, CreateRoleRequest, Permissions, Role, RoleId, UpdateRoleInput,
            UpdateRoleRequest,
        },
        ports::RoleService,
    },
    server_member::MemberId,
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
    let permissions =
        Permissions::try_from(request.permissions).map_err(|e| ApiError::BadRequest {
            msg: e.to_string(),
            error_code: None,
        })?;
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
    path = "/roles/{role_id}",
    tag = "roles",
    params(
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
    Path(role_id): Path<Uuid>,
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
    path = "/servers/{server_id}/roles",
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
    path = "/roles/{role_id}",
    tag = "roles",
    params(
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
    Path(role_id): Path<Uuid>,
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

#[utoipa::path(
    delete,
    path = "/roles/{role_id}",
    tag = "role",
    params(
        ("role_id" = String, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the server owner"),
        (status = 404, description = "Server or role not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_role(
    Path(role_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .delete_role(&RoleId(role_id))
        .await
        .map_err(Into::<ApiError>::into)?;
    Ok(Response::deleted(()))
}

#[utoipa::path(
     post,
     path = "/roles/{role_id}/members/{member_id}",
     tag = "role",
     params(
         ("role_id" = String, Path, description = "Role ID"),
         ("member_id" = String, Path, description = "Member ID")
     ),
     responses(
         (status = 201, description = "Role assigned successfully to member"),
         (status = 401, description = "Unauthorized"),
         (status = 403, description = "Forbidden - Not the server owner"),
         (status = 404, description = "Server or role not found"),
         (status = 500, description = "Internal server error")
     )
 )]
pub async fn assign_role(
    Path((role_id, member_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<MemberRole>, ApiError> {
    let member_role = state
        .service
        .assign_member_to_role(RoleId(role_id), MemberId(member_id))
        .await
        .map_err(Into::<ApiError>::into)?;
    Ok(Response::created(member_role))
}

#[utoipa::path(
     delete,
     path = "/roles/{role_id}/members/{member_id}",
     tag = "role",
     params(
         ("role_id" = String, Path, description = "Role ID"),
         ("member_id" = String, Path, description = "Member ID")
     ),
     responses(
         (status = 201, description = "Role unassigned successfully from member"),
         (status = 401, description = "Unauthorized"),
         (status = 403, description = "Forbidden - Not the server owner"),
         (status = 404, description = "Server or role not found"),
         (status = 500, description = "Internal server error")
     )
 )]
pub async fn unassign_role(
    Path((role_id, member_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .unassign_member_from_role(RoleId(role_id), MemberId(member_id))
        .await
        .map_err(Into::<ApiError>::into)?;
    Ok(Response::deleted(()))
}
