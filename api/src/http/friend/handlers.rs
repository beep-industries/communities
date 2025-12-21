use communities_core::domain::{
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

#[utoipa::path(
    get,
    path = "/friends",
    tag = "friends",
    params(
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of friends retrieved successfully", body = PaginatedResponse<Friend>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
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

#[utoipa::path(
    delete,
    path = "/friends/{friend_id}",
    tag = "friends",
    params(
        ("friend_id" = String, Path, description = "ID of the friend to delete")
    ),
    responses(
        (status = 200, description = "Friend deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Friend not found"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/friend-requests",
    tag = "friend-requests",
    params(
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of friend requests retrieved successfully", body = PaginatedResponse<FriendRequest>),
        (status = 401, description = "Unauthorized"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/friend-invitations",
    tag = "friend-invitations",
    params(
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of friend invitations retrieved successfully", body = PaginatedResponse<FriendRequest>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_friend_invitations(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<FriendRequest>>, ApiError> {
    let user_id = UserId::from(user_identity.user_id);

    let (friends, total) = state
        .service
        .get_friend_invitations(&pagination, &user_id)
        .await?;

    let response = PaginatedResponse {
        data: friends,
        total,
        page: pagination.page,
    };

    Ok(Response::ok(response))
}

#[utoipa::path(
    post,
    path = "/friend-requests",
    tag = "friend-requests",
    request_body = CreateFriendRequestInput,
    responses(
        (status = 201, description = "Friend request created successfully", body = FriendRequest),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Friend request already exists"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/friend-requests/accept",
    tag = "friend-requests",
    request_body = AcceptFriendRequestInput,
    responses(
        (status = 201, description = "Friend request accepted successfully", body = Friend),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Friend request not found"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/friend-requests/decline",
    tag = "friend-requests",
    request_body = DeclineFriendRequestInput,
    responses(
        (status = 201, description = "Friend request declined successfully", body = FriendRequest),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Friend request not found"),
    )
)]
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

#[utoipa::path(
    delete,
    path = "/friend-requests/{user_id_invited}",
    tag = "friend-requests",
    params(
        ("user_id_invited" = String, Path, description = "ID of the invited user whose friend request to delete")
    ),
    responses(
        (status = 200, description = "Friend request deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Friend request not found"),
    )
)]
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
