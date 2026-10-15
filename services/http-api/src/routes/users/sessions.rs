use axum::{Router, routing::post};

use crate::{handlers::users::*, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/refresh", post(refresh_token))
        .with_state(state)
}
