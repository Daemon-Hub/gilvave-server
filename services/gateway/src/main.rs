mod handler;
mod ws;

use axum::{Router, routing::get};
use mimalloc::MiMalloc;
use std::sync::Arc;
use tokio::net::TcpListener;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let realtime = Arc::new(gilvave_realtime::Realtime::default());

    let app = Router::new().route("/ws", get(move |ws| ws::ws_handler(ws, realtime.clone())));

    let listener = TcpListener::bind("0.0.0.0:3100").await?;
    println!("Running on http://localhost:3100");

    axum::serve(listener, app).await?;

    Ok(())
}
