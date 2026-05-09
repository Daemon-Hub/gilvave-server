mod events;
mod service;
mod state;
mod ws;

use axum::{Router, routing::get};
use mimalloc::MiMalloc;
use tokio::net::TcpListener;

use gilvave_infra::db::init_db;
use gilvave_messaging::start_consumer;
use gilvave_settings::setup_settings;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_settings();

    let state = state::AppState::new(init_db().await?).await;

    let queue_name = format!("gateway_events_{}", state.node_id);
    let mut rabbit_rx = start_consumer(state.broker.get_channel(), &queue_name).await;

    // Фоновая таска, слушает и забирает сообщения от Rabbit из mpsc канала
    let broadcast_state = state.clone();
    tokio::spawn(async move {
        println!("[Gateway] Started listening to RabbitMQ events");
        while let Some(json_str) = rabbit_rx.recv().await {
            let event: events::BrokerEvent = match serde_json::from_str(&json_str) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to parse broadcast event: {}", e);
                    continue;
                }
            };
            event.handle(&broadcast_state).await;
        }
    });

    let app = Router::new()
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3100").await?;
    println!("Running on ws://localhost:3100/ws");

    axum::serve(listener, app).await?;

    Ok(())
}
