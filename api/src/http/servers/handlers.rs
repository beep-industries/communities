use std::str::FromStr;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use beep_auth::Identity;
use communities_core::domain::{
    common::GetPaginated,
    friend::entities::UserId,
    server::{
        entities::{
            CreateServerRequest, Server, ServerId, ServerVisibility, UpdateServerRequest,
        },
        ports::ServerService,
    },
};
use uuid::Uuid;

use crate::http::server::{
    ApiError, AppState, Response,
    response::PaginatedResponse,
};

#[utoipa::path(
    post,
    path = "/servers",
    tag = "servers",
    request_body = CreateServerRequest,
    responses(
        (status = 201, description = "Server created successfully", body = Server),
        (status = 400, description = "Bad request - Invalid server name"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_server(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(request): Json<CreateServerRequest>,
) -> Result<Response<Server>, ApiError> {
    let input = request.into_input(UserId::from(identity.id().to_string()));
    let server = state.service.create_server(input).await?;
    Ok(Response::created(server))
}

#[utoipa::path(
    get,
    path = "/servers/{id}",
    tag = "servers",
    params(
        ("id" = String, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Server retrieved successfully", body = Server),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Server is private"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_server(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Server>, ApiError> {
    let server_id = ServerId::from(id);
    let server = state.service.get_server(&server_id).await?;

    // Only allow access to public servers or if user is the owner
    if server.visibility != ServerVisibility::Public && server.owner_id.0 != Uuid::from_str(&identity.id()).unwrap() {
        return Err(ApiError::Forbidden);
    }

    Ok(Response::ok(server))
}

#[utoipa::path(
    get,
    path = "/servers",
    tag = "servers",
    params(
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of servers retrieved successfully", body = PaginatedResponse<Server>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_servers(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<Server>>, ApiError> {
    let (servers, total) = state.service.list_servers(&pagination).await?;

    let response = PaginatedResponse {
        data: servers,
        total,
        page: pagination.page,
    };

    Ok(Response::ok(response))
}

#[utoipa::path(
    put,
    path = "/servers/{id}",
    tag = "servers",
    params(
        ("id" = String, Path, description = "Server ID")
    ),
    request_body = UpdateServerRequest,
    responses(
        (status = 200, description = "Server updated successfully", body = Server),
        (status = 400, description = "Bad request - Invalid server name"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the server owner"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_server(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(request): Json<UpdateServerRequest>,
) -> Result<Response<Server>, ApiError> {
    let server_id = ServerId::from(id);

    // Check if server exists and user is the owner
    let existing_server = state.service.get_server(&server_id).await?;
    if existing_server.owner_id.0 != Uuid::from_str(&identity.id()).unwrap() {
        return Err(ApiError::Forbidden);
    }

    let input = request.into_input(server_id);
    let server = state.service.update_server(input).await?;
    Ok(Response::ok(server))
}

#[utoipa::path(
    delete,
    path = "/servers/{id}",
    tag = "servers",
    params(
        ("id" = String, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Server deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the server owner"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_server(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    let server_id = ServerId::from(id);

    // Check if server exists and user is the owner
    let existing_server = state.service.get_server(&server_id).await?;
    if existing_server.owner_id.0 != Uuid::from_str(&identity.id()).unwrap() {
        return Err(ApiError::Forbidden);
    }

    state.service.delete_server(&server_id).await?;
    Ok(Response::deleted(()))
}
