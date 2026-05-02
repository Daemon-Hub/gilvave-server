use axum::{
    Json,
    extract::{Query, State},
};

use crate::{errors::AppError, state::AppState};
use gilvave_core::dto::server::*;
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
