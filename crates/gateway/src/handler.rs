use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use std::sync::Arc;
use tokio::sync::Mutex;

use gilvave_core::event::*;
use gilvave_infra::jwt::verify_jwt;
use gilvave_realtime::Realtime;

pub async fn handle_event(
    event: GatewayEvent,
    sender: &mut Arc<Mutex<SplitSink<WebSocket, Message>>>,
    rt: &Realtime,
) {
    match event {
        GatewayEvent::Dispatch(DispatchPayload::MessageCreate(msg)) => {
            // рассылаем всем
            rt.send(DispatchPayload::MessageCreate(msg));
        }

        GatewayEvent::Heartbeat => {
            let _ = send_event(sender, &GatewayEvent::HeartbeatAck).await;
        }

        GatewayEvent::Identify(payload) => {
            let claims = verify_jwt(&payload.token, "secret");

            if claims.is_err() {
                return;
            }

            let user_id = claims.unwrap().sub;

            tracing::info!("Authenticated user {}", user_id);
        }

        _ => {}
    }
}

pub async fn send_event(
    sender: &mut Arc<Mutex<SplitSink<WebSocket, Message>>>,
    event: &GatewayEvent,
) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string(event)?;
    sender.lock().await.send(Message::Text(json.into())).await?;
    Ok(())
}
