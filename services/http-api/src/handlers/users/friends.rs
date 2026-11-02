use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};

use gilvave_core::{
    dto::users::{auth::Claims, friends::*},
    error::CoreError,
    ids::UserId,
};

use crate::state::AppState;

pub async fn get_friends(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FriendView>>, CoreError> {
    let friends = state.users.friends.get_friends(claims.sub).await?;
    Ok(Json(friends))
}

pub async fn remove_friend(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(friend_id): Path<UserId>,
) -> Result<StatusCode, CoreError> {
    state
        .users
        .friends
        .remove_friend(claims.sub, friend_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_requests(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<FriendRequestsList>, CoreError> {
    let requests = state.users.friends.get_requests(claims.sub).await?;
    Ok(Json(requests))
}

pub async fn send_request(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<SendFriendRequest>,
) -> Result<(StatusCode, Json<SendFriendResponse>), CoreError> {
    let res = state.users.friends.send_request(claims.sub, body).await?;
    let status = match res.status {
        FriendRequestResult::Sent => StatusCode::CREATED,
        FriendRequestResult::Accepted => StatusCode::OK,
    };
    Ok((status, Json(res)))
}

pub async fn accept_request(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(sender_id): Path<UserId>,
) -> Result<StatusCode, CoreError> {
    state
        .users
        .friends
        .accept_request(claims.sub, sender_id)
        .await?;
    Ok(StatusCode::OK)
}

pub async fn decline_request(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, CoreError> {
    state
        .users
        .friends
        .decline_request(claims.sub, user_id)
        .await?;
    Ok(StatusCode::OK)
}

pub async fn get_blocked_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<BlockedUserView>>, CoreError> {
    let blocked = state.users.friends.get_blocked(claims.sub).await?;
    Ok(Json(blocked))
}

pub async fn block_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<BlockUserRequest>,
) -> Result<StatusCode, CoreError> {
    state
        .users
        .friends
        .block_user(claims.sub, body.user_id)
        .await?;
    Ok(StatusCode::OK)
}

pub async fn block_user_by_id(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, CoreError> {
    state.users.friends.block_user(claims.sub, user_id).await?;
    Ok(StatusCode::OK)
}

pub async fn unblock_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, CoreError> {
    state
        .users
        .friends
        .unblock_user(claims.sub, user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
