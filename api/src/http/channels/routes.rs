use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::server::AppState,
    http::channels::handlers::{
        __path_create_private_channel, __path_create_server_channel, __path_delete_channel,
        __path_get_channel, __path_list_channels, __path_update_channel,
        create_private_channel, create_server_channel, delete_channel, get_channel,
        list_channels, update_channel,
    },
};

pub fn channel_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_server_channel))
        .routes(routes!(create_private_channel))
        .routes(routes!(list_channels))
        .routes(routes!(get_channel))
        .routes(routes!(update_channel))
        .routes(routes!(delete_channel))
}

