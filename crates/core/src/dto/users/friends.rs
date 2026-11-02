use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{from_row, ids::UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendView {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

from_row!(FriendView, id, username, display_name, avatar, created_at);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestUser {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
}

from_row!(FriendRequestUser, id, username, display_name, avatar);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestView {
    pub user: FriendRequestUser,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

from_row!(FriendRequestView, user: FriendRequestUser, created_at);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FriendRequestsList {
    pub incoming: Vec<FriendRequestView>,
    pub outgoing: Vec<FriendRequestView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFriendRequest {
    pub user_id: Option<UserId>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FriendRequestResult {
    Sent,
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFriendResponse {
    pub status: FriendRequestResult,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockUserRequest {
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedUserView {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
}

from_row!(BlockedUserView, id, username, display_name, avatar);
