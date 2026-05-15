use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers::server::*, routes::channel, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_user_servers).post(create_server))
        .route("/all", get(get_all_public_servers))
        .route("/{server_id}/join_public", post(join_public))
        .route("/{server_id}/members", get(get_members))
        .with_state(state.clone())
        // Подгруппа для каналов
        .nest("/{server_id}/channels", channel::routes(state))
}
