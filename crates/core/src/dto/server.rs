use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{ServerId, UserId};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerView {
    pub id: ServerId,
    pub name: String,
    pub icon_url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCreateInfo {
    pub name: String,
    pub icon_url: Option<String>,
    pub is_public: bool,
}

#[derive(Deserialize)]
pub struct ServerFilters {
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct JoinInfo {
    pub server_id: ServerId,
    pub user_id: UserId,
}

#[derive(Serialize)]
pub struct Member {
    pub user_id: UserId,
}
