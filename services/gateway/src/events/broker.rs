use serde::{Deserialize, Serialize};

use crate::{events::ServerEvent, state::AppState};
use gilvave_core::{dto::message, ids::UserId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
pub enum BrokerEvent {
    // UserJoinedChannel { channel_id: ChannelId, user_id: UserId },
    // UserLeftChannel { channel_id: ChannelId, user_id: UserId },
    MessageCreated { message: message::MessageView },
    UserOnline { user_id: UserId },
}

impl BrokerEvent {
    pub async fn handle(self, state: &AppState) {
        match self {
            Self::MessageCreated { message } => {
                let server_event = ServerEvent::MessageNew {
                    payload: serde_json::to_value(&message).unwrap(),
                };

                let channels = state.channels.read().await;
                let users = state.users.read().await;

                for uid in channels.get(&message.channel_id).unwrap() {
                    users.get(uid).unwrap().send(server_event.clone()).ok();
                }
            }
            Self::UserOnline { user_id } => {}
        }
    }
}
