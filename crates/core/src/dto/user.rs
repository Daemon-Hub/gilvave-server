use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::ids::UserId;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct UserView {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub avatar: String,
}

pub struct Avatar {
    pub filename: String,
    pub bytes: Bytes,
}

pub struct UpdateAvatarInfo {
    pub user_id: UserId,
    pub url: String,
}

#[derive(Serialize)]
pub struct AvatarUrl {
    pub url: String,
}

pub struct BlacklistInfo {
    pub token: String,
    pub user_id: UserId,
}
