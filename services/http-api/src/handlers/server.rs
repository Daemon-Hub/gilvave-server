use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{errors::AppError, state::AppState};
use gilvave_core::{dto::server::*, ids::ServerId};
use gilvave_infra::security::auth::AuthUser;

pub async fn get_user_servers(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(filters): Query<ServerFilters>,
) -> Result<Json<Vec<ServerView>>, AppError> {
    let servers = match filters.role.as_deref() {
        Some("owner") => state.server_service.get_owned(user.id).await?,
        Some("member") => state.server_service.get_member(user.id).await?,
        _ => state.server_service.get_all_by_user(user.id).await?,
    };

    Ok(Json(servers))
}

pub async fn get_all_public_servers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ServerView>>, AppError> {
    Ok(Json(state.server_service.get_all_public().await?))
}

pub async fn create_server(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(info): Json<ServerCreateInfo>,
) -> Result<Json<ServerView>, AppError> {
    Ok(Json(state.server_service.create(info, user.id).await?))
}

/*
Если публичный сервер:
    юзер нажал кнопку -> отправил ид сервера -> юзер добавился на сервер
Если по ссылке приглашению:
    юзер перешел по ссылке -> проверить валидность ссылки (например, секретного токена) ->
    если секретный токен не истек или не удален -> добавить юзера
Если личное приглашение:
    админ выбрал юзера для добавления -> юзеру пришло личное приглашение ->
    если юзер принял приглашение -> добавить юзера
*/
pub async fn join_public(
    Path(server_id): Path<ServerId>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<(), AppError> {
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
) -> Result<Json<Vec<Member>>, AppError> {
    Ok(Json(state.server_service.get_members(server_id).await?))
}
