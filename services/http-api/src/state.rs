use axum::extract::FromRef;
use std::sync::Arc;

use gilvave_infra::{db::Database, service::*};
use gilvave_s3::S3;

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub session_service: SessionService,
    pub server_service: ServerService,
    pub channel_service: ChannelService,
    pub s3: S3,
}

impl AppState {
    pub async fn new(db: Arc<Database>) -> Self {
        Self {
            user_service: UserService { db: db.clone() },
            session_service: SessionService { db: db.clone() },
            server_service: ServerService { db: db.clone() },
            channel_service: ChannelService { db: db.clone() },
            s3: S3::new().await,
        }
    }
}

impl FromRef<AppState> for UserService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.user_service.clone()
    }
}
