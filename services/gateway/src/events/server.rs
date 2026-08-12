use serde::{Deserialize, Serialize};

use gilvave_core::dto::message::MessageView;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    HeartbeatAck,
    Hello,
    Error { message: String },
    MessageNew(MessageView),
    JoinSuccess,
    ChannelHistoryBefore(Vec<MessageView>),
    ChannelHistoryAfter(Vec<MessageView>),
}
