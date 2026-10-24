use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::{from_row, ids::ThemeId};

pub struct Avatar {
    pub filename: String,
    pub bytes: Bytes,
}

#[derive(Serialize)]
pub struct AvatarUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileView {
    pub name: String,
    pub bio: String,
    pub avatar: String,
    pub settings: Value,
}
from_row!(ProfileView, name, bio, avatar, settings);

pub type Profile = ProfileView;

#[derive(Serialize)]
pub struct Theme {
    pub id: ThemeId,
    pub name: String,
    pub author: String,
    pub description: String,
    pub config: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
from_row!(
    Theme,
    id,
    name,
    author,
    description,
    config,
    created_at,
    updated_at
);

#[derive(Deserialize)]
pub struct CreateThemeInfo {
    pub name: String,
    pub description: String,
    pub config: Value,
}
