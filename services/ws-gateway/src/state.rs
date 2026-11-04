use axum::extract::FromRef;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use gilvave_core::ids::{ChannelId, UserId};
use gilvave_infra::{
    db::Database,
    services::{FilesService, MessagesService, ServersService, UsersService},
};
use gilvave_messaging::IggyBroker;
use gilvave_redis::RedisService;

use crate::events::ServerEvent;

pub type Tx = mpsc::UnboundedSender<Arc<str>>;
pub type Dict<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct AppState {
    /// ID этого конкретного процесса (например, "gateway-1")
    pub node_id: Uuid,

    // Управляемые данные о пользователях и каналах
    pub users: Dict<UserId, Tx>,
    pub channels: Dict<ChannelId, HashSet<UserId>>,
    pub user_channels: Dict<UserId, HashSet<ChannelId>>,

    // Сервисы и клиенты
    pub redis: RedisService,
    pub broker: IggyBroker,
    pub users_service: UsersService,
    pub messages_service: MessagesService,
    pub servers_service: ServersService,
    pub files_service: FilesService,
}

impl AppState {
    pub async fn new(db: Arc<Database>) -> Self {
        let node_id = Uuid::new_v4();
        Self {
            node_id,
            users: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            user_channels: Arc::new(RwLock::new(HashMap::new())),
            redis: RedisService::new().await.unwrap(),
            broker: IggyBroker::new(&node_id.to_string()).await.unwrap(),
            users_service: UsersService::new(db.clone()),
            messages_service: MessagesService { db: db.clone() },
            servers_service: ServersService::new(db.clone()),
            files_service: FilesService::new(db),
        }
    }

    pub async fn broadcast_to_channel(&self, channel_id: ChannelId, server_event: ServerEvent) {
        let json = server_event.to_json();

        let (channels, users) = tokio::join!(self.channels.read(), self.users.read());

        if let Some(channel_users) = channels.get(&channel_id) {
            for uid in channel_users {
                if let Some(user_tx) = users.get(uid) {
                    user_tx.send(json.clone()).ok();
                }
            }
        }
    }
}

impl FromRef<AppState> for UsersService {
    fn from_ref(state: &AppState) -> Self {
        state.users_service.clone()
    }
}
