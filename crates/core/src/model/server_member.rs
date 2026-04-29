use crate::ids::*;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMember {
    pub server_id: ServerId,
    pub user_id: UserId,
    pub joined_at: UtcDateTime,
}
