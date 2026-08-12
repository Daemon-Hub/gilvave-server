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

                let channels = state.channels.read().await;
                let users = state.users.read().await;

                for uid in channels.get(&message.channel_id).unwrap() {
                    users.get(uid).unwrap().send(server_event.clone()).ok();
                }
            }
        }
    }
}
