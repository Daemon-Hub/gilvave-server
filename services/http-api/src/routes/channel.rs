use axum::{
    Router,
    routing::{get, patch},
};

use crate::{handlers::channel::*, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{channel_id}/name", patch(update_name))
        .route("/{channel_id}/position", patch(update_position))
        .with_state(state)
}
