use axum::extract::FromRef;
use std::{ops::Deref, sync::Arc};

use gilvave_infra::{db::Database, services::*};
use gilvave_s3::S3;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub users: UsersService,
    pub servers: ServersService,
    pub s3: S3,
}

impl AppState {
    pub async fn new(db: Arc<Database>) -> Self {
        let inner = AppStateInner {
            users: UsersService::new(db.clone()),
            servers: ServersService::new(db.clone()),
            s3: S3::new().await,
        };
        Self(Arc::new(inner))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for UsersService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.0.users.clone()
    }
}
