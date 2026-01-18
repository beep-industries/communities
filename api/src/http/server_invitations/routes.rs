use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::server::AppState,
    http::server_invitations::handlers::{
        __path_accept_invitation, __path_create_invitation, __path_get_invitation,
        accept_invitation, create_invitation, get_invitation,
    },
};

pub fn server_invitation_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_invitation))
        .routes(routes!(get_invitation))
        .routes(routes!(accept_invitation))
}
