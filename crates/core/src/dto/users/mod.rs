pub mod auth;
pub mod profile;
pub mod sessions;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{from_row, ids::UserId};
use profile::ProfileView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub joined_at: OffsetDateTime,
}
from_row!(User, id, username, email, password_hash, is_active, joined_at);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserView {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub joined_at: OffsetDateTime,
    pub profile: ProfileView,
}
