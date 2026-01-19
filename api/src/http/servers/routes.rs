use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::server::AppState,
    http::servers::handlers::{
        __path_create_server, __path_delete_server, __path_get_server, __path_update_server,
        create_server, delete_server, get_server, update_server,
    },
};

pub fn server_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_server))
        .routes(routes!(get_server))
        // .routes(routes!(list_servers))
        .routes(routes!(update_server))
        .routes(routes!(delete_server))
}
