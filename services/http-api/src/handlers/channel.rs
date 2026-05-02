use axum::{
    Json,
    extract::{Path, State},
};

use crate::{errors::AppError, state::AppState};
use gilvave_core::{
    dto::channel::*,
    ids::{ChannelId, ServerId, UserId},
};
use gilvave_infra::security::auth::AuthUser;

async fn with_channel_permissions<F, Fut>(
    state: &AppState,
    user_id: UserId,
    path: (ServerId, ChannelId),
    action: F,
) -> Result<View, AppError>
where
    F: FnOnce(ServerId, ChannelId) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<View>>,
{
    let (server_id, channel_id) = path;

    let is_owned = state
        .server_service
        .is_user_owned(user_id, server_id)
        .await
        .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    if !is_owned {
        return Err(AppError::Forbidden(
            "You do not own this server".to_string(),
        ));
    }

    action(server_id, channel_id).await.map_err(Into::into)
}

pub async fn get_all(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
) -> Result<Json<Vec<View>>, AppError> {
    let res = state
        .channel_service
        .get_server_channels(server_id)
        .await
        .map_err(|_| AppError::BadRequest("Error with get channels".to_string()))?;
    Ok(Json(res))
}

pub async fn create(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
    AuthUser(_): AuthUser,
    Json(info): Json<CreateInfo>,
) -> Result<Json<View>, AppError> {
    let res = state.channel_service.create(server_id, info).await?;
    Ok(Json(res))
}

pub async fn update_name(
    Path(path): Path<(ServerId, ChannelId)>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(name): Json<NameUpdate>,
) -> Result<Json<View>, AppError> {
    let res = with_channel_permissions(&state, user.id, path, |server_id, channel_id| {
        state
            .channel_service
            .update_name(server_id, channel_id, name)
    })
    .await?;

    Ok(Json(res))
}

pub async fn update_position(
    Path(path): Path<(ServerId, ChannelId)>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(position): Json<PositionUpdate>,
) -> Result<Json<View>, AppError> {
    let res = with_channel_permissions(&state, user.id, path, |server_id, channel_id| {
        state
            .channel_service
            .update_position(server_id, channel_id, position)
    })
    .await?;

    Ok(Json(res))
}
