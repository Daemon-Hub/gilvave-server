use axum::{
    Json,
    extract::{Path, State},
};

use crate::state::AppState;
use gilvave_core::{dto::server::*, error::CoreError, ids::ServerId};
use gilvave_infra::security::auth::AuthUser;

pub async fn get_user_servers(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<ServerSmallPart>>, CoreError> {
    let servers = state.server_service.retrieve_user_servers(user.id).await?;
    Ok(Json(servers))
}

pub async fn get_server_by_id(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Server>, CoreError> {
    Ok(Json(
        state.server_service.get_by_id(server_id, user.id).await?,
    ))
}

pub async fn get_public_servers(
    Path(page): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<(Vec<Server>, bool)>, CoreError> {
    Ok(Json(
        state.server_service.get_public((page - 1) * 20).await?,
    ))
}

pub async fn create_server(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(info): Json<ServerCreateInfo>,
) -> Result<Json<Server>, CoreError> {
    Ok(Json(state.server_service.create(info, user.id).await?))
}

/*
Если публичный сервер:
    юзер нажал кнопку -> отправил ид сервера -> юзер добавился на сервер
Если по ссылке приглашению:
    юзер перешел по ссылке -> проверить валидность ссылки (например, секретного токена) ->
    если секретный токен не истек или не удален -> добавить юзера
*/
pub async fn join_public(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<(), CoreError> {
    state
        .server_service
        .add_user(JoinInfo {
            server_id,
            user_id: user.id,
        })
        .await?;
    Ok(())
}

pub async fn get_members(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
    AuthUser(_): AuthUser,
) -> Result<Json<Vec<MemberView>>, CoreError> {
    Ok(Json(state.server_service.get_members(server_id).await?))
}
