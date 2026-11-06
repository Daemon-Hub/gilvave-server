use serde::{Deserialize, Serialize};

use crate::{events::ServerEvent, state::AppState};
use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId, ServerId, UserId},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[allow(clippy::enum_variant_names)]
pub enum BrokerEvent {
    MessageCreated {
        message: MessageView,
    },
    MessageUpdated {
        message: MessageView,
    },
    MessageDeleted {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    UserOnline {
        user_id: UserId,
        server_ids: Vec<ServerId>,
    },
    UserOffline {
        user_id: UserId,
        server_ids: Vec<ServerId>,
    },
}

impl BrokerEvent {
    pub async fn handle(self, state: &AppState) {
        match self {
            Self::MessageCreated { message } => {
                let channel_id = message.channel_id;
                let server_event = ServerEvent::MessageCreate(message);
                state.broadcast_to_channel(channel_id, server_event).await;
            }
            Self::MessageUpdated { message } => {
                let channel_id = message.channel_id;
                let server_event = ServerEvent::MessageUpdate(message);
                state.broadcast_to_channel(channel_id, server_event).await;
            }
            Self::MessageDeleted {
                channel_id,
                message_id,
            } => {
                let server_event = ServerEvent::MessageDelete {
                    channel_id,
                    message_id,
                };
                state.broadcast_to_channel(channel_id, server_event).await;
            }
            Self::UserOnline {
                user_id,
                server_ids,
            } => match state.redis.get_users_online(&server_ids).await {
                Ok(users) => {
                    let server_event = ServerEvent::UserOnline { user_id };
                    state.broadcast_to_users(&users, server_event).await;
                }
                Err(e) => tracing::error!("[WS] Failed to collect user servers online status: {e}"),
            },
            Self::UserOffline {
                user_id,
                server_ids,
            } => match state.redis.get_users_online(&server_ids).await {
                Ok(users) => {
                    let server_event = ServerEvent::UserOffline { user_id };
                    state.broadcast_to_users(&users, server_event).await;
                }
                Err(e) => tracing::error!("[WS] Failed to collect user servers online status: {e}"),
            },
        }
    }
}
