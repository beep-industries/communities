use core::domain::{common::GetPaginated, friend::{entities::{Friend, UserId}, ports::FriendService}};

use axum::{Extension, extract::{Query, State}};

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