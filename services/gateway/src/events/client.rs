use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    events::{BrokerEvent, EventHandler, ServerEvent},
    state::AppState,
};
use gilvave_core::{
    dto::{
        messages::{CreateInfo, GetHistoryInfo},
        users::User,
    },
    ids::{ChannelId, MessageId},
    validation::validate_message,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    Heartbeat,
    MessageCreate {
        channel_id: ChannelId,
        content: String,
        #[serde(default)]
        reply_to_id: Option<MessageId>,
        #[serde(default)]
        forwarded_from: Option<serde_json::Value>,
    },
    #[serde(alias = "MESSAGE_EDIT")]
    MessageUpdate {
        channel_id: ChannelId,
        message_id: MessageId,
        content: String,
    },
    MessageDelete {
        channel_id: ChannelId,
        message_id: MessageId,
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
    async fn handle(self, state: AppState, user: User, sender: UnboundedSender<Arc<str>>) {
        match self {
            Self::Heartbeat => {
                _ = sender.send(ServerEvent::HeartbeatAck.to_json());
            }
            Self::MessageCreate {
                channel_id,
                content,
                reply_to_id,
                forwarded_from,
            } => {
                let sanitized_content = match validate_message(&content) {
                    Ok(s) => s,
                    Err(err_msg) => {
                        _ = sender.send(ServerEvent::Error {
                            message: err_msg.into(),
                        }.to_json());
                        return;
                    }
                };

                match state
                    .messages_service
                    .create(CreateInfo {
                        channel_id,
                        author_id: user.id,
                        author_name: user.username,
                        content: sanitized_content,
                        reply_to_id,
                        forwarded_from,
                    })
                    .await
                {
                    Ok(message_view) => {
                        let broker_event = BrokerEvent::MessageCreated {
                            message: message_view,
                        };

                        if let Err(e) = state.broker.publish(&broker_event).await {
                            tracing::error!("[Apache Iggy] Publish error: {}", e);
                            _ = sender.send(ServerEvent::Error {
                                message: "Failed to broadcast".into(),
                            }.to_json());
                        }
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to save message to DB: {}", e);
                        _ = sender.send(ServerEvent::Error {
                            message: "Failed to send message".into(),
                        }.to_json());
                    }
                }
            }
            Self::MessageUpdate {
                channel_id,
                message_id,
                content,
            } => {
                let sanitized_content = match validate_message(&content) {
                    Ok(s) => s,
                    Err(err_msg) => {
                        _ = sender.send(ServerEvent::Error {
                            message: err_msg.into(),
                        }.to_json());
                        return;
                    }
                };

                match state
                    .messages_service
                    .update(message_id, channel_id, user.id, sanitized_content)
                    .await
                {
                    Ok(message_view) => {
                        let broker_event = BrokerEvent::MessageUpdated {
                            message: message_view,
                        };

                        if let Err(e) = state.broker.publish(&broker_event).await {
                            tracing::error!("[Apache Iggy] Publish error: {}", e);
                            _ = sender.send(ServerEvent::Error {
                                message: "Failed to broadcast".into(),
                            }.to_json());
                        }
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to update message in DB: {}", e);
                        _ = sender.send(ServerEvent::Error {
                            message: "Failed to update message".into(),
                        }.to_json());
                    }
                }
            }
            Self::MessageDelete {
                channel_id,
                message_id,
            } => {
                match state
                    .messages_service
                    .delete(message_id, channel_id, user.id)
                    .await
                {
                    Ok(deleted) => {
                        if deleted {
                            let broker_event = BrokerEvent::MessageDeleted {
                                channel_id,
                                message_id,
                            };

                            if let Err(e) = state.broker.publish(&broker_event).await {
                                tracing::error!("[Apache Iggy] Publish error: {}", e);
                                _ = sender.send(ServerEvent::Error {
                                    message: "Failed to broadcast".into(),
                                }.to_json());
                            }
                        } else {
                            _ = sender.send(ServerEvent::Error {
                                message: "Message not found or forbidden".into(),
                            }.to_json());
                        }
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to delete message from DB: {}", e);
                        _ = sender.send(ServerEvent::Error {
                            message: "Failed to delete message".into(),
                        }.to_json());
                    }
                }
            }
            Self::JoinChannel { channel_id } => {
                let (mut channels, mut user_channels) = tokio::join!(
                    state.channels.write(),
                    state.user_channels.write()
                );
                channels
                    .entry(channel_id)
                    .or_default()
                    .insert(user.id);
                user_channels
                    .entry(user.id)
                    .or_default()
                    .insert(channel_id);

                let count = channels.get(&channel_id).map_or(0, |u| u.len());
                _ = sender.send(ServerEvent::JoinSuccess.to_json());
                tracing::info!(
                    "[WS] User {} joined channel {channel_id}. Total online in channel: {count}",
                    user.id
                );
            }
            Self::LeftChannel { channel_id } => {
                let (mut channels, mut user_channels) = tokio::join!(
                    state.channels.write(),
                    state.user_channels.write()
                );
                if let Some(users) = channels.get_mut(&channel_id) {
                    users.remove(&user.id);
                    tracing::info!(
                        "[WS] User {} left channel {channel_id}. Total online in channel: {}",
                        user.id,
                        users.len()
                    );
                }
                if let Some(chans) = user_channels.get_mut(&user.id) {
                    chans.remove(&channel_id);
                }
            }
            Self::ChannelHistoryBefore {
                channel_id,
                timestamp,
            } => {
                let history = match state
                    .messages_service
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
                _ = sender.send(ServerEvent::ChannelHistoryBefore(history).to_json());
            }
            Self::ChannelHistoryAfter {
                channel_id,
                timestamp,
            } => {
                let history = match state
                    .messages_service
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
                _ = sender.send(ServerEvent::ChannelHistoryAfter(history).to_json());
            }
        }
    }
}
