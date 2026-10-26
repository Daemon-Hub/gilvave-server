use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    from_row,
    ids::{ChannelId, MessageId, UserId},
};

#[derive(Debug, Deserialize)]
pub struct CreateInfo {
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub author_name: String,
    pub content: String,
    #[serde(default)]
    pub reply_to_id: Option<MessageId>,
    #[serde(default)]
    pub forwarded_from: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageView {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: Option<UserId>,
    pub author_name: String,
    pub content: String,
    #[serde(default)]
    pub edited: bool,
    #[serde(default)]
    pub reply_to_id: Option<MessageId>,
    #[serde(default)]
    pub forwarded_from: Option<serde_json::Value>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Deserialize)]
pub struct GetHistoryInfo {
    pub channel_id: ChannelId,
    pub timestamp: OffsetDateTime,
}

from_row!(
    MessageView,
    id,
    channel_id,
    author_id,
    author_name,
    content,
    edited,
    reply_to_id,
    forwarded_from,
    created_at,
);
