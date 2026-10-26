use serde::{Deserialize, Serialize};

use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    HeartbeatAck,
    Hello,
    Error { message: String },
    MessageNew(MessageView),
    MessageUpdate(MessageView),
    MessageDelete {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    JoinSuccess,
    ChannelHistoryBefore(Vec<MessageView>),
    ChannelHistoryAfter(Vec<MessageView>),
}
