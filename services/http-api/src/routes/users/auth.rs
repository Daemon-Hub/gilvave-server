use axum::{Router, routing::post};

use crate::{handlers::users::*, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .with_state(state.clone())
}
