use axum::routing::{delete, get, post};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    friend::handlers::{
        __path_get_friends, accept_friend_request, create_friend_request, decline_friend_request,
        delete_friend, delete_friend_request, get_friend_requests, get_friends,
    },
    server::AppState,
};

pub fn friend_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_friends))
        .route("/friends/{friend_id}", delete(delete_friend))
        .route("/friends/requests", get(get_friend_requests))
        .route("/friends/requests", post(create_friend_request))
        .route("/friends/requests/accept", post(accept_friend_request))
        .route("/friends/requests/decline", post(decline_friend_request))
        .route(
            "/friends/requests/{user_id_invited}",
            delete(delete_friend_request),
        )
}
