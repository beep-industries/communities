use core::domain::{
    common::GetPaginated,
    friend::{
        entities::{
            AcceptFriendRequestInput, CreateFriendRequestInput, DeclineFriendRequestInput,
            DeleteFriendInput, Friend, FriendRequest, UserId,
        },
        ports::{FriendRequestService, FriendService},
    },
};

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::http::server::{
    ApiError, AppState, Response, middleware::auth::entities::UserIdentity,
    response::PaginatedResponse,
};

pub async fn get_friends(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<Friend>>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);

    let (friends, total) = state.service.get_friends(&pagination, &user_id).await?;

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
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);
    let friend_id = UserId::from(friend_id);

    state
        .service
        .delete_friend(DeleteFriendInput {
            user_id_1: user_id,
            user_id_2: friend_id,
        })
        .await?;

    Ok(Response::deleted(()))
}

pub async fn get_friend_requests(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<FriendRequest>>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);

    let (friends, total) = state
        .service
        .get_friend_requests(&pagination, &user_id)
        .await?;

    let response = PaginatedResponse {
        data: friends,
        total,
        page: pagination.page,
    };

    Ok(Response::ok(response))
}

pub async fn create_friend_request(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(input): Json<CreateFriendRequestInput>,
) -> Result<Response<FriendRequest>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);
    let friend_request = state
        .service
        .create_friend_request(&user_id, &input.user_id_invited)
        .await?;
    Ok(Response::created(friend_request))
}

pub async fn accept_friend_request(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(input): Json<AcceptFriendRequestInput>,
) -> Result<Response<Friend>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);
    let friend = state
        .service
        .accept_friend_request(&input.user_id_requested, &user_id)
        .await?;
    Ok(Response::created(friend))
}

pub async fn decline_friend_request(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(input): Json<DeclineFriendRequestInput>,
) -> Result<Response<FriendRequest>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);
    let friend_request = state
        .service
        .decline_friend_request(&input.user_id_requested, &user_id)
        .await?;
    Ok(Response::created(friend_request))
}

pub async fn delete_friend_request(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Path(user_id_invited): Path<Uuid>,
) -> Result<Response<()>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);
    let user_id_invited = UserId::from(user_id_invited);
    state
        .service
        .delete_friend_request(&user_id, &user_id_invited)
        .await?;
    Ok(Response::deleted(()))
}
