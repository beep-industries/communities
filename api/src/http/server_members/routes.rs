use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::server::AppState,
    http::server_members::handlers::{
        __path_delete_member, __path_list_members, __path_update_member, delete_member,
        list_members, update_member,
    },
};

pub fn server_member_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_members))
        .routes(routes!(update_member))
        .routes(routes!(delete_member))
}
