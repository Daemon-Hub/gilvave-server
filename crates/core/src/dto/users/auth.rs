use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserInfo {
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

#[derive(Serialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}
