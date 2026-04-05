use crate::ids::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User<'a> {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
}
