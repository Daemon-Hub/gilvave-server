use axum::{routing::post, Router};
use sqlx::PgPool;
use std::sync::Arc;

use crate::handlers::user::*;

pub fn routes(pool: Arc<PgPool>) -> Router {
    Router::new()
        .nest(
            "/users",
            Router::new()
                .route("/register", post(register))
                .route("/login", post(login))
                .with_state(pool.clone()),
        )
        .with_state(pool)
}
