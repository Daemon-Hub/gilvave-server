mod server;
mod user;
mod channel;

use axum::Router;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/users", user::routes(state.clone()))
        .nest("/servers", server::routes(state.clone()))
}
