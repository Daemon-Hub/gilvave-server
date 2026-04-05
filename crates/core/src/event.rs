use crate::ids::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
pub enum GatewayEvent {
    Hello(HelloPayload),
    Heartbeat,
    HeartbeatAck,
    Identify(IdentifyPayload),
    Dispatch(DispatchPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayload {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyPayload {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum DispatchPayload {
    Ready(ReadyEvent),
    MessageCreate(MessageCreateEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyEvent {
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreateEvent {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub content: String,
}
