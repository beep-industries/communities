use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use communities_core::domain::{
    common::GetPaginated,
    server::{
        entities::{
            CreateServerRequest, SearchServerQuery, Server, ServerId, ServerVisibility,
            UpdateServerRequest,
        },
        ports::ServerService,
    },
};
use uuid::Uuid;

use crate::http::server::{
    ApiError, AppState, Response, middleware::auth::entities::UserIdentity,
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
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<CreateServerRequest>,
) -> Result<Response<Server>, ApiError> {
    let input = request.into_input(*user_identity);
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
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Server>, ApiError> {
    let server_id = ServerId::from(id);
    let server = state.service.get_server(&server_id).await?;

    user_identity.can_view_server(server_id).await?;

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
    Extension(_user_identity): Extension<UserIdentity>,
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
    get,
    path = "/servers/@me",
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
pub async fn list_user_servers(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<Server>>, ApiError> {
    let (servers, total) = state
        .service
        .list_user_servers(&pagination, *user_identity)
        .await?;

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
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<UpdateServerRequest>,
) -> Result<Response<Server>, ApiError> {
    let server_id = ServerId::from(id);

    // Check if server exists and user is the owner
    let server = state.service.get_server(&server_id).await?;

    user_identity.can_manage_server(server.id).await?;

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
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let server_id = ServerId::from(id);

    // Check if server exists and user is the owner
    let existing_server = state.service.get_server(&server_id).await?;
    if existing_server.owner_id != user_identity.user_id {
        return Err(ApiError::Forbidden);
    }

    state.service.delete_server(&server_id).await?;
    Ok(Response::deleted(()))
}

#[utoipa::path(
    get,
    path = "/servers/search",
    tag = "servers",
    params(
        ("q" = Option<String>, Query, description = "Search query for server name (optional - returns random servers if not provided, max 100 chars)"),
        GetPaginated
    ),
    responses(
        (status = 200, description = "Servers retrieved successfully", body = PaginatedResponse<Server>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_or_discover_servers(
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
    Query(search): Query<SearchServerQuery>,
) -> Result<Response<PaginatedResponse<Server>>, ApiError> {
    let sanitized_query = search.sanitized_query();
    let safe_pagination = search.safe_pagination();

    let (servers, total) = state
        .service
        .search_or_discover(sanitized_query, &safe_pagination)
        .await?;

    let response = PaginatedResponse {
        data: servers,
        total,
        page: safe_pagination.page,
    };

    Ok(Response::ok(response))
}
