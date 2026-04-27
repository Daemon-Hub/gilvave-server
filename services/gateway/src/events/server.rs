use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};

use super::EventHandler;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    HeartbeatAck,
    Hello {
        heartbeat_interval: u64,
    },
}

#[async_trait::async_trait]
impl EventHandler for ServerEvent {
    async fn handle(self, state: AppState, sender: &mut SplitSink<WebSocket, Message>) {}
}
