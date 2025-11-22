use axum::{Router, routing::{delete, get}};

use crate::http::{friend::handlers::{delete_friend, get_friends}, server::AppState};

pub fn friend_routes() -> Router<AppState> {
    Router::new()
        .route("/friends", get(get_friends))
        .route("/friends/{friend_id}", delete(delete_friend))
}