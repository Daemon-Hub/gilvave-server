use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{ChannelId, MessageId, UserId};

#[derive(Debug, Deserialize)]
pub struct CreateInfo {
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub author_name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageView {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: Option<UserId>,
    pub author_name: String,
    pub content: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Deserialize)]
pub struct GetHistoryInfo {
    pub channel_id: ChannelId,
    pub from: OffsetDateTime,
}