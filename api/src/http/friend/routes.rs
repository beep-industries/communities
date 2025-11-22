use axum::{Router, routing::get};

use crate::http::{friend::handlers::get_friends, server::AppState};

pub fn friend_routes() -> Router<AppState> {
    Router::new()
        .route("/friends", get(get_friends))
}