use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use time::OffsetDateTime;

use crate::{
    from_row,
    ids::{SessionId, UserId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub avatar: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
    pub device_info: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct SessionCreateInfo<'a> {
    pub user_id: UserId,
    pub refresh_token_hash: &'a [u8],
    pub device_info: serde_json::Value,
    pub ip_address: IpAddr,
}

#[derive(Serialize)]
pub struct SessionCheckInfo {
    pub id: SessionId,
    pub user_id: UserId,
    pub expires_at: OffsetDateTime,
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

from_row!(User, id, username, email, password_hash, is_active, avatar);
from_row!(SessionCheckInfo, id, user_id, expires_at);
