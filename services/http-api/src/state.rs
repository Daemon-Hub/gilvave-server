use axum::extract::FromRef;
use sqlx::PgPool;

use gilvave_infra::service::*;

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub ref_token_service: RefTokenService,
    pub server_service: ServerService,
    pub channel_service: ChannelService,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            user_service: UserService { db: db.clone() },
            ref_token_service: RefTokenService { db: db.clone() },
            server_service: ServerService { db: db.clone() },
            channel_service: ChannelService { db: db.clone() },
        }
    }
}

impl FromRef<AppState> for UserService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.user_service.clone()
    }
}
