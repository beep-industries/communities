use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    friend::handlers::{
        __path_accept_friend_request, __path_create_friend_request, __path_decline_friend_request,
        __path_delete_friend, __path_delete_friend_request, __path_get_friend_invitations,
        __path_get_friend_requests, __path_get_friends, accept_friend_request,
        create_friend_request, decline_friend_request, delete_friend, delete_friend_request,
        get_friend_invitations, get_friend_requests, get_friends,
    },
    server::AppState,
};

pub fn friend_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_friends))
        .routes(routes!(delete_friend))
        .routes(routes!(get_friend_requests))
        .routes(routes!(get_friend_invitations))
        .routes(routes!(create_friend_request))
        .routes(routes!(accept_friend_request))
        .routes(routes!(decline_friend_request))
        .routes(routes!(delete_friend_request))
}
