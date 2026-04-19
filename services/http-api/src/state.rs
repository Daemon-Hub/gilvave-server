use sqlx::PgPool;
use axum::extract::FromRef;

use gilvave_infra::service::{ref_token_service::RefTokenService, user_service::UserService};

#[derive(Clone)]
pub struct AppState {
    // pub db: PgPool,
    pub user_service: UserService,
    pub ref_token_service: RefTokenService,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            // db: db.clone(),
            user_service: UserService { db: db.clone() },
            ref_token_service: RefTokenService { db: db.clone() },
        }
    }
}

impl FromRef<AppState> for UserService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.user_service.clone()
    }
}
