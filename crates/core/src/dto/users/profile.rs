use bytes::Bytes;
use serde::Serialize;

use crate::ids::UserId;

pub struct Avatar {
    pub filename: String,
    pub bytes: Bytes,
}

pub struct UpdateAvatarInfo {
    pub user_id: UserId,
    pub url: String,
}

#[derive(Serialize)]
pub struct AvatarUrl {
    pub url: String,
}
