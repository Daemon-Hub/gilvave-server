mod events;
mod service;
mod state;
mod ws;

use axum::{Router, routing::get};
use mimalloc::MiMalloc;
use tokio::net::TcpListener;

use gilvave_infra::db::init_db;
use gilvave_settings::setup_settings;
use state::AppState;
use ws::ws_handler;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_settings();
    
    let state = AppState::new(init_db().await?).await;

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3100").await?;
    println!("Running on ws://localhost:3100/ws");

    axum::serve(listener, app).await?;

    Ok(())
}
