mod handlers;
mod routes;
mod state;
mod errors;

use jsonwebtoken::crypto::{CryptoProvider, rust_crypto::DEFAULT_PROVIDER};
use mimalloc::MiMalloc;
use tokio::net::TcpListener;

use gilvave_infra::db::init_db;
use gilvave_settings::setup_settings;

use crate::state::AppState;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt().init();
    // dotenvy::dotenv()?;
    setup_settings();
    CryptoProvider::install_default(&DEFAULT_PROVIDER).unwrap();

    let state = AppState::new(init_db().await?);

    let app = routes::routes(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
