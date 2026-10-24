use serde::{Deserialize, Serialize};

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
