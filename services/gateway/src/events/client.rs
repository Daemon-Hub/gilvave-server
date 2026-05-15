use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};

use crate::{
    events::{BrokerEvent, EventHandler, ServerEvent},
    state::AppState,
};
use gilvave_core::{
    dto::message::{CreateInfo, GetHistoryInfo},
    ids::{ChannelId},
    model::User,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    Heartbeat,
    MessageCreate {
        channel_id: ChannelId,
        content: String,
    },
    JoinChannel {
        channel_id: ChannelId,
    },
    LeftChannel {
        channel_id: ChannelId,
    },
    ChannelHistory {
        channel_id: ChannelId,
        #[serde(with = "time::serde::rfc3339")]
        from: time::OffsetDateTime,
    },
}

#[async_trait::async_trait]
impl EventHandler for ClientEvent {
    async fn handle(self, state: AppState, user: User, sender: &mut SplitSink<WebSocket, Message>) {
        match self {
            Self::Heartbeat => {
                let json = serde_json::to_string(&ServerEvent::HeartbeatAck).unwrap();
                _ = sender.send(Message::Text(json.into())).await;
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
                            _ = sender
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
                        _ = sender.send(Message::Text(json.into())).await;
                    }
                }
            }
            Self::JoinChannel { channel_id } => {
                let mut channels = state.channels.write().await;
                channels
                    .entry(channel_id)
                    .and_modify(|users| {
                        users.insert(user.id);
                    })
                    .or_insert([user.id].into());
                let json = serde_json::to_string(&ServerEvent::JoinSuccess).unwrap();
                _ = sender.send(Message::Text(json.into())).await;
                println!(
                    "[WS] User {} joined to channel {channel_id}. Total online in channel: {}",
                    user.id,
                    channels.get(&channel_id).iter().len()
                );
            }
            Self::LeftChannel { channel_id } => {
                let mut channels = state.channels.write().await;
                channels.entry(channel_id).and_modify(|users| {
                    users.remove(&user.id);
                    println!(
                        "[WS] User {} left in channel {channel_id}. Total online in channel: {}",
                        user.id,
                        users.len()
                    );
                });
            }
            Self::ChannelHistory { channel_id, from } => {
                let history = state
                    .message_service
                    .get_history_by_time(GetHistoryInfo { channel_id, from })
                    .await
                    .unwrap();
                let json = serde_json::to_string(&ServerEvent::ChannelHistory(history)).unwrap();
                _ = sender.send(Message::Text(json.into())).await;
            }
        }
    }
}
