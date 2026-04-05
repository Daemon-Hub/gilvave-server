use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use gilvave_core::event::*;
use gilvave_realtime::Realtime;

use crate::handler::{handle_event, send_event};

pub async fn ws_handler(ws: WebSocketUpgrade, rt: Arc<Realtime>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, rt))
}

async fn handle_socket(socket: WebSocket, rt: Arc<Realtime>) {
    let mut rx = rt.subscribe();

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    
    // задача на отправку событий клиенту
    let mut sender1 = Arc::clone(&sender);
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let gateway_event = GatewayEvent::Dispatch(event);
            let _ = send_event(&mut sender1, &gateway_event).await;
        }
    });

    // чтение клиента
    let mut sender2 = Arc::clone(&sender);
    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Ok(event) = serde_json::from_str::<GatewayEvent>(&text) {
                handle_event(event, &mut sender2, &rt).await;
            }
        }
    }
}
