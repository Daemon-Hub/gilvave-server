mod events;
mod service;
mod state;
mod ws;

use axum::{Router, routing::get};
use mimalloc::MiMalloc;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Semaphore};

use gilvave_infra::db::init_db;
use gilvave_messaging::start_consumer;
use gilvave_settings::setup_settings;

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

    let db = Arc::new(init_db().await?);
    db.warmup(5).await;

    let state = state::AppState::new(db).await;

    let queue_name = format!("gateway_events_{}", state.node_id);
    let mut rabbit_rx = start_consumer(state.broker.get_channel(), &queue_name).await;

    // Ограничиваем параллелизм — backpressure + контроль за пулом БД
    let concurrency = std::sync::Arc::new(Semaphore::new(100));
    let broadcast_state = state.clone();

    // Параллельная обработка событий с backpressure
    tokio::spawn(async move {
        tracing::info!("[RabbitMQ] Started listening events");

        while let Some(json_str) = rabbit_rx.recv().await {
            let permit = concurrency.clone().acquire_owned().await.unwrap();
            let state = broadcast_state.clone();

            tokio::spawn(async move {
                let _permit = permit;
                let event: events::BrokerEvent = match serde_json::from_str(&json_str) {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("[RabbitMQ] Failed to parse broadcast event: {e}");
                        return;
                    }
                };
                event.handle(&state).await;
            });
        }
    });

    let app = Router::new()
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3100").await?;
    tracing::info!("[WS] Running on ws://localhost:3100/ws");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap();
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("[WS] Shutdown signal received");
}
