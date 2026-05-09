use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::ServerId;

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
