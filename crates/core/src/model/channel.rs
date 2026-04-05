use serde::{Serialize, Deserialize};
use crate::ids::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub guild_id: GuildId,
    pub name: String,
}