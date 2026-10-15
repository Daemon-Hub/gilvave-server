use axum::{
    Router,
    routing::{get, patch},
};

use crate::{handlers::users::*, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/me", get(get_profile))
        .route("/me/avatar", patch(update_avatar))
        .with_state(state)
}
