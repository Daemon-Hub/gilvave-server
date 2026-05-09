use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};

use crate::{
    events::{BrokerEvent, EventHandler, ServerEvent},
    state::AppState,
};
use gilvave_core::{dto::message::CreateInfo, ids::ChannelId, model::User};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    Heartbeat,
    MessageCreate {
        channel_id: ChannelId,
        content: String,
    },
}

#[async_trait::async_trait]
impl EventHandler for ClientEvent {
    async fn handle(self, state: AppState, user: User, sender: &mut SplitSink<WebSocket, Message>) {
        match self {
            Self::Heartbeat => {
                let ack = ServerEvent::HeartbeatAck;
                let json = serde_json::to_string(&ack).unwrap();
                let _ = sender.send(Message::Text(json.into())).await;
            }
            Self::MessageCreate {
                channel_id,
                content,
            } => {
                match state
                    .message_service
                    .create(CreateInfo {
                        channel_id,
                        author_id: user.id,
                        author_name: user.username,
                        content,
                    })
                    .await
                {
                    Ok(message_view) => {
                        let broker_event = BrokerEvent::MessageCreated {
                            message: message_view,
                        };

                        if let Err(e) = state.broker.publish(&broker_event).await {
                            eprintln!("RabbitMQ publish error: {}", e);
                            let _ = sender
                                .send(Message::Text(
                                    serde_json::to_string(&ServerEvent::Error {
                                        message: "Failed to broadcast".into(),
                                    })
                                    .unwrap()
                                    .into(),
                                ))
                                .await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to save message to DB: {}", e);
                        let error_event = ServerEvent::Error {
                            message: "Failed to send message".into(),
                        };
                        let json = serde_json::to_string(&error_event).unwrap();
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                }
            }
        }
    }
}
