use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers::user::*, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest(
            "/users",
            Router::new()
                .route("/register", post(register))
                .route("/login", post(login))
                .route("/refresh", post(refresh_token))
                .route("/me", get(get_profile)),
        )
        .with_state(state)
}
