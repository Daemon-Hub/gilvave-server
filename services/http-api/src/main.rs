mod handlers;
mod routes;
mod state;

use jsonwebtoken::crypto::{CryptoProvider, rust_crypto::DEFAULT_PROVIDER};
use mimalloc::MiMalloc;
use std::sync::Arc;
use tokio::net::TcpListener;

use gilvave_infra::db::init_db;
use gilvave_settings::setup_settings;

use crate::state::AppState;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .compact()
        .init();

    setup_settings();
    CryptoProvider::install_default(&DEFAULT_PROVIDER).unwrap();

    let db = Arc::new(init_db().await?);
    db.warmup(5).await;

    let state = AppState::new(db).await;

    let app = routes::routes(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("[REST-API] Running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
