use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    from_row,
    ids::{ServerId, UserId},
};

#[derive(Serialize, Deserialize)]
pub struct ServerSmallPart {
    pub id: ServerId,
    pub name: String,
    pub icon_url: String,
}
from_row!(ServerSmallPart, id, name, icon_url);

#[derive(Serialize)]
pub struct Server {
    pub id: ServerId,
    pub owner_id: UserId,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub cover: String,
    pub is_public: bool,
    pub members_count: u32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
from_row!(
    Server,
    id,
    owner_id,
    name,
    description,
    icon_url,
    cover,
    is_public,
    members_count,
    created_at
);

#[derive(Serialize, Deserialize)]
pub struct ServerCreateInfo {
    pub name: String,
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
pub struct MemberView {
    pub user_id: UserId,
    pub username: String,
    pub avatar: String,
}
from_row!(MemberView, user_id, username, avatar);
