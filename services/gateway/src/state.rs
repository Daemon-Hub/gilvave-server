use axum::extract::FromRef;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::events::ServerEvent;
use crate::service::SessionService;
use gilvave_core::ids::UserId;
use gilvave_infra::service::user_service::UserService;
use gilvave_messaging::RabbitClient;

type Tx = mpsc::UnboundedSender<ServerEvent>;

#[derive(Clone)]
pub struct AppState {
    /// ID этого конкретного процесса (например, "gateway-1")
    pub node_id: Uuid,
    pub users: Arc<RwLock<HashMap<UserId, Tx>>>,
    pub session: SessionService,
    pub rabbit: RabbitClient,
    pub user_service: UserService,
}

impl AppState {
    pub async fn new(db: PgPool) -> Self {
        Self {
            node_id: Uuid::new_v4(),
            users: Arc::new(RwLock::new(HashMap::new())),
            session: SessionService::new(),
            rabbit: RabbitClient::new().await.unwrap(),
            user_service: UserService { db },
        }
    }
}

impl FromRef<AppState> for UserService {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}
