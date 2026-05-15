use serde::{Deserialize, Serialize};

use gilvave_core::dto::message::MessageView;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    HeartbeatAck,
    Hello { heartbeat_interval: u64 },
    Error { message: String },
    MessageNew(MessageView),
    JoinSuccess,
    ChannelHistory(Vec<MessageView>),
}
