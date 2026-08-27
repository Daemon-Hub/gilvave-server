use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    events::{BrokerEvent, EventHandler, ServerEvent},
    state::AppState,
};
use gilvave_core::{
    dto::message::{CreateInfo, GetHistoryInfo},
    dto::user::User,
    ids::ChannelId,
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
    ChannelHistoryBefore {
        channel_id: ChannelId,
        #[serde(with = "time::serde::rfc3339")]
        timestamp: time::OffsetDateTime,
    },
    ChannelHistoryAfter {
        channel_id: ChannelId,
        #[serde(with = "time::serde::rfc3339")]
        timestamp: time::OffsetDateTime,
    },
}

#[async_trait::async_trait]
impl EventHandler for ClientEvent {
    async fn handle(self, state: AppState, user: User, sender: UnboundedSender<ServerEvent>) {
        match self {
            Self::Heartbeat => {
                _ = sender.send(ServerEvent::HeartbeatAck);
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
                            tracing::error!("[RabbitMQ] Publish error: {}", e);
                            _ = sender.send(ServerEvent::Error {
                                message: "Failed to broadcast".into(),
                            });
                        }
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to save message to DB: {}", e.to_string());
                        _ = sender.send(ServerEvent::Error {
                            message: "Failed to send message".into(),
                        });
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
                _ = sender.send(ServerEvent::JoinSuccess);
                tracing::info!(
                    "[WS] User {} joined to channel {channel_id}. Total online in channel: {}",
                    user.id,
                    channels.get(&channel_id).iter().len()
                );
            }
            Self::LeftChannel { channel_id } => {
                let mut channels = state.channels.write().await;
                channels.entry(channel_id).and_modify(|users| {
                    users.remove(&user.id);
                    tracing::info!(
                        "[WS] User {} left in channel {channel_id}. Total online in channel: {}",
                        user.id,
                        users.len()
                    );
                });
            }
            Self::ChannelHistoryBefore {
                channel_id,
                timestamp,
            } => {
                let history = match state
                    .message_service
                    .get_history_before(GetHistoryInfo {
                        channel_id,
                        timestamp,
                    })
                    .await
                {
                    Ok(hist) => hist,
                    Err(e) => {
                        tracing::error!("[DB] Failed to get history before: {}", e);
                        return;
                    }
                };
                _ = sender.send(ServerEvent::ChannelHistoryBefore(history));
            }
            Self::ChannelHistoryAfter {
                channel_id,
                timestamp,
            } => {
                let history = match state
                    .message_service
                    .get_history_after(GetHistoryInfo {
                        channel_id,
                        timestamp,
                    })
                    .await
                {
                    Ok(hist) => hist,
                    Err(e) => {
                        tracing::error!("[DB] Failed to get history after: {}", e);
                        return;
                    }
                };
                _ = sender.send(ServerEvent::ChannelHistoryAfter(history));
            }
        }
    }
}
