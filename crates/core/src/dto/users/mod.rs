pub mod auth;
pub mod profile;
pub mod sessions;

use serde::{Deserialize, Serialize};

use crate::{from_row, ids::UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub avatar: String,
    pub bio: String,
}
from_row!(User, id, username, email, password_hash, is_active, avatar, bio);

#[derive(Serialize)]
pub struct UserView {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub avatar: String,
    pub bio: String,
}
