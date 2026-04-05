use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

mod handlers;
mod routes;

pub fn router(pool: Arc<PgPool>) -> Router {
    routes::routes(pool)
}
