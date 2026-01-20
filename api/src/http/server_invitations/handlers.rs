use axum::{
    Extension, Json,
    extract::{Path, State},
};
use communities_core::domain::{
    friend::entities::UserId,
    server::{entities::ServerId, ports::ServerService},
    server_invitation::{
        entities::{
            AcceptInvitationInput, CreateServerInvitationRequest, ServerInvitation,
            ServerInvitationId,
        },
        ports::ServerInvitationService,
    },
};
use uuid::Uuid;

use crate::http::server::{ApiError, AppState, Response, middleware::auth::entities::UserIdentity};

#[utoipa::path(
    post,
    path = "/servers/{server_id}/invitations",
    tag = "server_invitations",
    request_body = CreateServerInvitationRequest,
    params(
        ("server_id" = String, Path, description = "Server ID")
    ),
    responses(
        (status = 201, description = "Invitation created successfully", body = ServerInvitation),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not server owner"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_invitation(
    Path(server_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<CreateServerInvitationRequest>,
) -> Result<Response<ServerInvitation>, ApiError> {
    let server_id = ServerId::from(server_id);
    let inviter_id = UserId::from(user_identity.user_id);

    // Verify user owns the server
    let server = state.service.get_server(&server_id).await?;
    if server.owner_id != inviter_id {
        return Err(ApiError::Forbidden);
    }

    let input = request.into_input(server_id, inviter_id);
    let invitation = state.service.create_invitation(input).await?;
    Ok(Response::created(invitation))
}

#[utoipa::path(
    get,
    path = "/invitations/{invitation_id}",
    tag = "server_invitations",
    params(
        ("invitation_id" = String, Path, description = "Invitation ID")
    ),
    responses(
        (status = 200, description = "Invitation retrieved successfully", body = ServerInvitation),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_invitation(
    Path(invitation_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
) -> Result<Response<ServerInvitation>, ApiError> {
    let invitation_id = ServerInvitationId::from(invitation_id);
    let invitation = state.service.get_invitation(&invitation_id).await?;
    Ok(Response::ok(invitation))
}

#[utoipa::path(
    post,
    path = "/invitations/{invitation_id}/accept",
    tag = "server_invitations",
    params(
        ("invitation_id" = String, Path, description = "Invitation ID")
    ),
    responses(
        (status = 200, description = "Invitation accepted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the invitee or invitation expired"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn accept_invitation(
    Path(invitation_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let accept_input = AcceptInvitationInput {
        invitation_id: ServerInvitationId::from(invitation_id),
        user_id: *user_identity,
    };

    state.service.accept_invitation(&accept_input).await?;
    Ok(Response::ok(()))
}
