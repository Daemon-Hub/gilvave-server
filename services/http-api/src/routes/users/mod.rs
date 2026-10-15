mod auth;
mod profile;
mod sessions;

use axum::Router;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth::routes(state.clone()))
        .nest("/profile", profile::routes(state.clone()))
        .nest("/sessions", sessions::routes(state.clone()))
}
