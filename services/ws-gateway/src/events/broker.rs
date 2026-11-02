use serde::{Deserialize, Serialize};

use crate::{events::ServerEvent, state::AppState};
use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId},
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
        }
    }
}
