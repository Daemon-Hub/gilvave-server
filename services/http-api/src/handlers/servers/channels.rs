use axum::{
    Extension, Json,
    extract::{Path, State},
};

use crate::state::AppState;
use gilvave_core::{
    dto::{servers::channels::*, users::User},
    error::{CoreError, DatabaseError},
    ids::{ChannelId, ServerId, UserId},
};

async fn with_channel_permissions<F, Fut>(
    state: &AppState,
    user_id: UserId,
    path: (ServerId, ChannelId),
    action: F,
) -> Result<ChannelView, CoreError>
where
    F: FnOnce(ServerId, ChannelId) -> Fut,
    Fut: std::future::Future<Output = Result<ChannelView, DatabaseError>>,
{
    let (server_id, channel_id) = path;

    let is_owned = state.servers.is_user_owned(user_id, server_id).await?;

    if !is_owned {
        return Err(CoreError::Forbidden(
            "You do not own this server".to_string(),
        ));
    }

    action(server_id, channel_id).await.map_err(Into::into)
}

pub async fn get_all(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ChannelView>>, CoreError> {
    Ok(Json(
        state
            .servers
            .channels
            .get_server_channels(server_id)
            .await?,
    ))
}

pub async fn create(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
    Json(info): Json<CreateInfo>,
) -> Result<Json<ChannelView>, CoreError> {
    Ok(Json(state.servers.channels.create(server_id, info).await?))
}

pub async fn update_name(
    Path(path): Path<(ServerId, ChannelId)>,
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(name): Json<NameUpdate>,
) -> Result<Json<ChannelView>, CoreError> {
    let res = with_channel_permissions(&state, user.id, path, |server_id, channel_id| {
        state
            .servers
            .channels
            .update_name(server_id, channel_id, name)
    })
    .await?;

    Ok(Json(res))
}

pub async fn update_position(
    Path(path): Path<(ServerId, ChannelId)>,
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(position): Json<PositionUpdate>,
) -> Result<Json<ChannelView>, CoreError> {
    let res = with_channel_permissions(&state, user.id, path, |server_id, channel_id| {
        state
            .servers
            .channels
            .update_position(server_id, channel_id, position)
    })
    .await?;

    Ok(Json(res))
}
