use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use super::{EventHandler, ServerEvent};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    Heartbeat,
    MessageCreate { content: String },
}

#[async_trait::async_trait]
impl EventHandler for ClientEvent {
    async fn handle(self, state: AppState, sender: &mut SplitSink<WebSocket, Message>) {
        match self {
            ClientEvent::Heartbeat => {
                let ack = ServerEvent::HeartbeatAck;
                let json = serde_json::to_string(&ack).unwrap();
                let _ = sender.send(Message::Text(json.into())).await;
            }

            ClientEvent::MessageCreate { content } => {
                let payload = serde_json::json!({
                    "content": content,
                    "timestamp": UtcDateTime::now(),
                });

                if let Err(e) = state
                    .rabbit
                    .publish(&payload)
                    .await
                {
                    eprintln!("RabbitMQ error: {}", e);
                }
            }
        }
    }
}
