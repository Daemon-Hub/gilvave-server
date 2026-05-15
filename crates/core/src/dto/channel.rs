use serde::{Deserialize, Serialize};

use crate::ids::*;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "channel_type", rename_all = "lowercase")]
pub enum ChannelType {
    TEXT,
    VOICE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInfo {
    pub name: String,
    pub r#type: ChannelType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameUpdate(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub old: i32,
    pub new: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelView {
    pub id: ChannelId,
    pub name: String,
    pub r#type: ChannelType,
    pub position: i32,
}
