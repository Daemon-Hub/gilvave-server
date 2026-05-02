use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use crate::dto::channel::ChannelType;
use crate::ids::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub owner_id: UserId,
    pub icon_url: String,
    pub created_at: UtcDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMember {
    pub server_id: ServerId,
    pub user_id: UserId,
    pub joined_at: UtcDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
    pub r#type: ChannelType,
    pub position: u16,
    pub created_at: UtcDateTime,
}
