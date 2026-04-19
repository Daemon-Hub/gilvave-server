use crate::ids::{RefreshTokenId, UserId};
use serde::{Deserialize, Serialize};
use time::UtcDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: RefreshTokenId,
    pub user_id: UserId,
    pub token: String,
    pub expires_at: UtcDateTime,
    pub created_at: UtcDateTime,
}
