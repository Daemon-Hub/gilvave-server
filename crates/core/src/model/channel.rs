use crate::ids::*;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
    pub r#type: ChannelType,
    pub position: u16,
    pub topic: String,
    pub created_at: UtcDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChannelType {
    TEXT,
    VOICE,
}
