use crate::ids::{SessionId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,
    pub sid: SessionId,
    pub exp: i64,
    pub jti: Uuid,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
}

/// Проверенный паспорт пользователя, инжектируемый API Gateway в заголовок X-User-Data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayUserData {
    pub id: UserId,
    pub session_id: SessionId,
    pub jti: Uuid,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserInfo {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl CreateUserInfo {
    pub fn to_slice(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
    pub fn from_slice(v: Vec<u8>) -> Self {
        serde_json::from_slice(&v).unwrap()
    }
}

#[derive(Deserialize)]
pub struct ConfirmParams {
    pub token: String,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
    pub device_info: serde_json::Value,
}

#[derive(Serialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}
