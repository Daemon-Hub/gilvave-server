use serde::{Serialize, Deserialize};
use crate::ids::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Guild {
    pub id: GuildId,
    pub name: String,
    pub owner_id: UserId,
}