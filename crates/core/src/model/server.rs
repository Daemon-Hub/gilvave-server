use crate::ids::*;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;


#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub owner_id: UserId,
    pub icon_url: String,
    pub created_at: UtcDateTime
}
