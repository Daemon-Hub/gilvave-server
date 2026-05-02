use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::dispatch_event;
use crate::events::{ClientEvent, EventHandler, ServerEvent};
use crate::state::AppState;

use gilvave_core::ids::UserId;
use gilvave_infra::security::auth::AuthUser;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user.id, state))
}

pub async fn handle_socket(socket: WebSocket, user_id: UserId, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    if let Err(e) = state
        .session
        .set_user_online(user_id, &state.node_id.to_string())
        .await
    {
        println!("{}", e);
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

    {
        let mut users = state.users.write().await;
        users.insert(user_id, tx.clone());
        println!(
            "[WS] User {} connected. Total online: {}",
            user_id.0,
            users.len()
        );
    }

    let hello = ServerEvent::Hello {
        heartbeat_interval: 30000,
    };

    if let Err(e) = sender
        .send(Message::Text(serde_json::to_string(&hello).unwrap().into()))
        .await
    {
        println!("[WS] Failed to send Hello: {}", e);
        cleanup_user(user_id, &state).await.ok();
        return;
    }

    let sender = Arc::new(Mutex::new(sender));

    let sender_recv = sender.clone();
    let recv_task = async {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.into_text() {
                let mut sender = sender_recv.lock().await;
                handle_event(text.to_string(), state.clone(), &mut sender).await;
            }
        }
    };

    let sender_send = sender.clone();
    let send_task = async {
        while let Some(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            let mut sender = sender_send.lock().await;
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = recv_task => { println!("[WS] Receiver task ended for user {}", user_id.0); },
        _ = send_task => { println!("[WS] Sender task ended for user {}", user_id.0); },
    };

    cleanup_user(user_id, &state).await.ok();
}

async fn cleanup_user(user_id: UserId, state: &AppState) -> anyhow::Result<()> {
    state.users.write().await.remove(&user_id);
    state.session.remove_user(user_id).await?;
    println!("[WS] User {} cleaned up", user_id.0);
    Ok(())
}

pub async fn handle_event(
    text: String,
    state: AppState,
    sender: &mut SplitSink<WebSocket, Message>,
) {
    dispatch_event!(&text, state, sender, [ClientEvent,]);
}
