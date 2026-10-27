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
    services::{MessagesService, ServersService, UsersService},
};
use gilvave_messaging::RabbitClient;
use gilvave_redis::RedisService;

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
    pub broker: RabbitClient,
    pub users_service: UsersService,
    pub messages_service: MessagesService,
    pub servers_service: ServersService,
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
            broker: RabbitClient::new(&node_id.to_string()).await.unwrap(),
            users_service: UsersService::new(db.clone()),
            messages_service: MessagesService { db: db.clone() },
            servers_service: ServersService::new(db),
        }
    }
}

impl FromRef<AppState> for UsersService {
    fn from_ref(state: &AppState) -> Self {
        state.users_service.clone()
    }
}
