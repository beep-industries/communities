use crate::http::server::{ApiError, AppState, Response, middleware::auth::entities::UserIdentity};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use communities_core::domain::{
    authorization::ports::AuthorizationService,
    channel::{
        entities::{
            Channel, ChannelId, CreatePrivateChannelRequest, CreateServerChannelRequest,
            UpdateChannelRequest,
        },
        ports::ChannelService,
    },
    server::{entities::ServerId, ports::ServerService},
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/servers/{server_id}/channels",
    tag = "channels",
    params(
        ("server_id" = String, Path, description = "Server ID")
    ),
    request_body = CreateServerChannelRequest,
    responses(
        (status = 201, description = "Server channel created successfully", body = Channel),
        (status = 400, description = "Bad request - Invalid channel data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not authorized to create channel in this server"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_server_channel(
    Path(server_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<CreateServerChannelRequest>,
) -> Result<Response<Channel>, ApiError> {
    // Verify server exists
    state.service.get_server(&ServerId::from(server_id)).await?;

    state
        .service
        .check_authz(
            user_identity.user_id,
            beep_authz::Permissions::ManageServer,
            beep_authz::SpiceDbObject::Server(server_id.to_string()),
        )
        .await?;
    // TODO: Check if user has permission to create channels in this server
    let input = request.into_input(ServerId::from(server_id));

    let channel = state.service.create_server_channel(input).await?;
    Ok(Response::created(channel))
}

#[utoipa::path(
    post,
    path = "/channels",
    tag = "channels",
    request_body = CreatePrivateChannelRequest,
    responses(
        (status = 201, description = "Private channel created successfully", body = Channel),
        (status = 400, description = "Bad request - Invalid channel data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_private_channel(
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
    Json(request): Json<CreatePrivateChannelRequest>,
) -> Result<Response<Channel>, ApiError> {
    let input = request.into_input();
    let channel = state.service.create_private_channel(input).await?;
    Ok(Response::created(channel))
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}/channels",
    tag = "channels",
    params(
        ("server_id" = String, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "List of channels retrieved successfully", body = Vec<Channel>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_channels(
    Path(server_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<Channel>>, ApiError> {
    // Verify server exists
    state.service.get_server(&ServerId::from(server_id)).await?;

    let channels = state
        .service
        .list_channels_in_server(ServerId::from(server_id))
        .await?;

    Ok(Response::ok(channels))
}

#[utoipa::path(
    get,
    path = "/channels/{id}",
    tag = "channels",
    params(
        ("id" = String, Path, description = "Channel ID")
    ),
    responses(
        (status = 200, description = "Channel retrieved successfully", body = Channel),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Channel not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_channel(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<Channel>, ApiError> {
    let channel_id = ChannelId::from(id);
    let channel = state.service.get_channel_by_id(channel_id).await?;

    // TODO: Check if user has permission to view this channel

    Ok(Response::ok(channel))
}

#[utoipa::path(
    put,
    path = "/channels/{id}",
    tag = "channels",
    params(
        ("id" = String, Path, description = "Channel ID")
    ),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Channel updated successfully", body = Channel),
        (status = 400, description = "Bad request - Invalid channel data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not authorized to update this channel"),
        (status = 404, description = "Channel not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_channel(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
    Json(request): Json<UpdateChannelRequest>,
) -> Result<Response<Channel>, ApiError> {
    let channel_id = ChannelId::from(id);

    // TODO: Check if user has permission to update this channel

    let input = request.into_input(channel_id);
    let channel = state.service.update_channel(input).await?;
    Ok(Response::ok(channel))
}

#[utoipa::path(
    delete,
    path = "/channels/{id}",
    tag = "channels",
    params(
        ("id" = String, Path, description = "Channel ID")
    ),
    responses(
        (status = 200, description = "Channel deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not authorized to delete this channel"),
        (status = 404, description = "Channel not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_channel(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let channel_id = ChannelId::from(id);

    // TODO: Check if user has permission to delete this channel

    state.service.delete_channel(channel_id).await?;
    Ok(Response::deleted(()))
}
