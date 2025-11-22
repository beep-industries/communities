use core::domain::{common::GetPaginated, friend::{entities::{DeleteFriendInput, Friend, UserId}, ports::FriendService}};

use axum::{Extension, extract::{Path, Query, State}};
use uuid::Uuid;

use crate::http::server::{ApiError, AppState, Response, middleware::AuthState, response::PaginatedResponse};

pub async fn get_friends(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Query(pagination): Query<GetPaginated>
) -> Result<Response<PaginatedResponse<Friend>>, ApiError> {
    let user_id = UserId::from(auth_state.user_id);

    let (friends, total) = state
        .service
        .get_friends(&pagination, &user_id)
        .await?;

    let response = PaginatedResponse {
        data: friends,
        total,
        page: pagination.page,
    };

    Ok(Response::ok(response))
}

pub async fn delete_friend(
    Path(friend_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>
) -> Result<Response<()>, ApiError> {
    let user_id = UserId::from(auth_state.user_id);
    let friend_id = UserId::from(friend_id);

    state.service.delete_friend(DeleteFriendInput { user_id_1: user_id, user_id_2: friend_id }).await?;

    Ok(Response::ok(()))
}