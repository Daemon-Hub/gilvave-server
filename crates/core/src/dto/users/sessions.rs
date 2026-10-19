use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use crate::{
    from_row,
    ids::{SessionId, UserId},
};

#[derive(Serialize, Deserialize)]
pub struct SessionCreateInfo<'a> {
    pub user_id: UserId,
    pub refresh_token_hash: &'a [u8],
    pub device_info: serde_json::Value,
    pub ip_address: IpAddr,
}

#[derive(Serialize)]
pub struct SessionIds {
    pub id: SessionId,
    pub user_id: UserId,
}
from_row!(SessionIds, id, user_id);

#[derive(Debug, Serialize)]
pub struct SessionView {
    pub id: SessionId,
    pub device_info: serde_json::Value,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub last_used_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}
from_row!(
    SessionView,
    id,
    device_info,
    ip_address,
    location,
    last_used_at,
    created_at
);

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub struct BlacklistInfo {
    pub token: String,
    pub user_id: UserId,
}
