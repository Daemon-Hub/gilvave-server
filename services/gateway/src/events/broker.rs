use serde::{Deserialize, Serialize};

use crate::{events::ServerEvent, state::AppState};
use gilvave_core::dto::message::MessageView;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
pub enum BrokerEvent {
    MessageCreated { message: MessageView },
}

impl BrokerEvent {
    pub async fn handle(self, state: &AppState) {
        match self {
            Self::MessageCreated { message } => {
                let server_event = ServerEvent::MessageNew(message.clone());

                let (channels, users) = tokio::join!(state.channels.read(), state.users.read());

                if let Some(channel_users) = channels.get(&message.channel_id) {
                    for uid in channel_users {
                        if let Some(user_tx) = users.get(uid) {
                            user_tx.send(server_event.clone()).ok();
                        }
                    }
                }
            }
        }
    }
}
