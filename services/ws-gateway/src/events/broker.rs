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
                let server_event = ServerEvent::MessageNew(message);
                let json = server_event.to_json();

                let (channels, users) = tokio::join!(state.channels.read(), state.users.read());

                if let Some(channel_users) = channels.get(&channel_id) {
                    for uid in channel_users {
                        if let Some(user_tx) = users.get(uid) {
                            user_tx.send(json.clone()).ok();
                        }
                    }
                }
            }
            Self::MessageUpdated { message } => {
                let channel_id = message.channel_id;
                let server_event = ServerEvent::MessageUpdate(message);
                let json = server_event.to_json();

                let (channels, users) = tokio::join!(state.channels.read(), state.users.read());

                if let Some(channel_users) = channels.get(&channel_id) {
                    for uid in channel_users {
                        if let Some(user_tx) = users.get(uid) {
                            user_tx.send(json.clone()).ok();
                        }
                    }
                }
            }
            Self::MessageDeleted {
                channel_id,
                message_id,
            } => {
                let server_event = ServerEvent::MessageDelete {
                    channel_id,
                    message_id,
                };
                let json = server_event.to_json();

                let (channels, users) = tokio::join!(state.channels.read(), state.users.read());

                if let Some(channel_users) = channels.get(&channel_id) {
                    for uid in channel_users {
                        if let Some(user_tx) = users.get(uid) {
                            user_tx.send(json.clone()).ok();
                        }
                    }
                }
            }
        }
    }
}
