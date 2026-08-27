use axum::extract::FromRef;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::events::ServerEvent;
use crate::service::RedisService;
use gilvave_core::ids::{ChannelId, UserId};
use gilvave_infra::{
    db::Database,
    service::{MessageService, ServerService, UserService},
};
use gilvave_messaging::RabbitClient;

type Tx = mpsc::UnboundedSender<ServerEvent>;
type Dict<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct AppState {
    /// ID этого конкретного процесса (например, "gateway-1")
    pub node_id: Uuid,

    // Управляемые данные о пользователях и каналах
    pub users: Dict<UserId, Tx>,
    // ===================================================================================
    // TODO: закончить с онлайном и разобраться в максимально эффективной рассылке статуса
    // ===================================================================================
    pub channels: Dict<ChannelId, HashSet<UserId>>,

    // Сервисы и клиенты
    pub redis: RedisService,
    pub broker: RabbitClient,
    pub user_service: UserService,
    pub message_service: MessageService,
    pub server_service: ServerService,
}

impl AppState {
    pub async fn new(db: Arc<Database>) -> Self {
        let node_id = Uuid::new_v4();
        Self {
            node_id,
            users: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            redis: RedisService::new(),
            broker: RabbitClient::new(&node_id.to_string()).await.unwrap(),
            user_service: UserService { db: db.clone() },
            message_service: MessageService { db: db.clone() },
            server_service: ServerService { db: db.clone() },
        }
    }
}

impl FromRef<AppState> for UserService {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}
