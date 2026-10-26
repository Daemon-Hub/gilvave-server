use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    events::{BrokerEvent, EventHandler, ServerEvent},
    state::AppState,
};
use gilvave_core::{
    validation::validate_message,
    dto::{
        messages::{CreateInfo, GetHistoryInfo},
        users::User,
    },
    ids::{ChannelId, MessageId},
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
    async fn handle(self, state: AppState, user: User, sender: UnboundedSender<ServerEvent>) {
        match self {
            Self::Heartbeat => {
                _ = sender.send(ServerEvent::HeartbeatAck);
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
                        });
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
                        });
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
                            tracing::error!("[RabbitMQ] Publish error: {}", e);
                            _ = sender.send(ServerEvent::Error {
                                message: "Failed to broadcast".into(),
                            });
                        }
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to update message in DB: {}", e.to_string());
                        _ = sender.send(ServerEvent::Error {
                            message: "Failed to update message".into(),
                        });
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
                                tracing::error!("[RabbitMQ] Publish error: {}", e);
                                _ = sender.send(ServerEvent::Error {
                                    message: "Failed to broadcast".into(),
                                });
                            }
                        } else {
                            _ = sender.send(ServerEvent::Error {
                                message: "Message not found or forbidden".into(),
                            });
                        }
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to delete message from DB: {}", e.to_string());
                        _ = sender.send(ServerEvent::Error {
                            message: "Failed to delete message".into(),
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
                _ = sender.send(ServerEvent::ChannelHistoryBefore(history));
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
                _ = sender.send(ServerEvent::ChannelHistoryAfter(history));
            }
        }
    }
}
