mod servers;
mod users;

use axum::Router;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/users", users::routes(state.clone()))
        .nest("/servers", servers::routes(state.clone()))
}
