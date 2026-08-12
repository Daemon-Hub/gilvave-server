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

use gilvave_core::{dto::user::User, ids::UserId};
use gilvave_infra::security::auth::AuthUser;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user, state))
}

pub async fn handle_socket(ws: WebSocket, user: User, state: AppState) {
    let (mut sender, mut receiver) = ws.split();

    if let Err(e) = state
        .session
        .set_user_online(user.id, &state.node_id.to_string())
        .await
    {
        tracing::error!("[Redis] Failed to set user online: {}", e);
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

    {
        let mut users = state.users.write().await;
        users.insert(user.id, tx.clone());
        tracing::info!(
            "[WS] User {} connected. Total online: {}",
            user.id.0,
            users.len()
        );
    }

    let hello = ServerEvent::Hello;

    if let Err(e) = sender
        .send(Message::Text(serde_json::to_string(&hello).unwrap().into()))
        .await
    {
        tracing::error!("[WS] Failed to send Hello: {}", e);
        cleanup_user(user.id, &state).await.ok();
        return;
    }

    let sender = Arc::new(Mutex::new(sender));

    let sender_recv = sender.clone();
    let recv_task = async {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.into_text() {
                let mut sender = sender_recv.lock().await;
                handle_event(text.to_string(), state.clone(), user.clone(), &mut sender).await;
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
        _ = recv_task => { tracing::info!("[WS] Receiver task ended for user {}", user.id.0); },
        _ = send_task => { tracing::info!("[WS] Sender task ended for user {}", user.id.0); },
    };

    cleanup_user(user.id, &state).await.ok();
}

pub async fn handle_event(
    text: String,
    state: AppState,
    user: User,
    sender: &mut SplitSink<WebSocket, Message>,
) {
    dispatch_event!(&text, state, user, sender, [ClientEvent,]);
}

async fn cleanup_user(
    user_id: UserId,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    state.users.write().await.remove(&user_id);
    state
        .channels
        .write()
        .await
        .iter_mut()
        .for_each(|(_, users)| {
            users.remove(&user_id);
        });
    state.session.remove_user(user_id).await?;
    tracing::info!("[WS] User {} cleaned up", user_id.0);
    Ok(())
}
