use axum::{extract::State, Json};
use sqlx::PgPool;
use std::sync::Arc;

use gilvave_core::dto::user::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use gilvave_infra::auth::{hash_password, verify_password};
use gilvave_infra::jwt::create_jwt;
use gilvave_infra::user_repo;

pub async fn register(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    let hash = hash_password(&payload.password).unwrap();

    let user = user_repo::create_user(&pool, &payload.username, &hash)
        .await
        .unwrap();

    Json(user)
}

pub async fn login(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<LoginRequest>,
) -> Json<LoginResponse> {
    let user = user_repo::find_by_username(&pool, &payload.username)
        .await
        .unwrap();

    if !verify_password(&user.password_hash, &payload.password) {
        panic!("invalid password");
    }

    Json(LoginResponse {
        token: create_jwt(&user.id.0.to_string(), "secret").unwrap(),
    })
}
