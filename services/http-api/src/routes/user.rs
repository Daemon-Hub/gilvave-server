use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::{handlers::user::*, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_profile))
        .route("/me/avatar", patch(update_avatar))
        .with_state(state)
}
