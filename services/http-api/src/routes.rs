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
                .route("/foo", get(foo)),
        )
        .with_state(state)
}
