use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    http::role::handlers::{
        __path_assign_role, __path_create_role, __path_delete_role, __path_get_role,
        __path_list_members_by_role, __path_list_roles_by_server, __path_unassign_role,
        __path_update_role, assign_role, create_role, delete_role, get_role,
        list_members_by_role, list_roles_by_server, unassign_role, update_role,
    },
};

pub fn role_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_role))
        .routes(routes!(get_role))
        .routes(routes!(update_role))
        .routes(routes!(list_roles_by_server))
        .routes(routes!(delete_role))
        .routes(routes!(assign_role))
        .routes(routes!(unassign_role))
        .routes(routes!(list_members_by_role))
}
