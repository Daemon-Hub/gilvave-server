use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::dispatch_event;
use crate::events::{ClientEvent, EventHandler, ServerEvent};
use crate::state::AppState;

use gilvave_core::{dto::users::User, ids::{ServerId, UserId}};
use gilvave_infra::security::auth::AuthUser;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    AuthUser(user, _): AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user, state))
}

pub async fn handle_socket(ws: WebSocket, user: User, state: AppState) {
    let (mut sender, mut receiver) = ws.split();

    let hello_json = ServerEvent::Hello.to_json();

    if let Err(e) = sender
        .send(Message::Text(hello_json.as_ref().into()))
        .await
    {
        tracing::error!("[WS] Failed to send Hello: {}", e);
        return;
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<Arc<str>>();

    tokio::join!(
        async {
            let mut users = state.users.write().await;
            users.insert(user.id, tx.clone());
            tracing::info!(
                "[WS] User {} connected. Total online: {}",
                user.id.0,
                users.len()
            );
        },
        async {
            let server_ids: Vec<ServerId> = state
                .servers_service
                .retrieve_user_servers(user.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.id)
                .collect();

            state
                .redis
                .set_user_online(user.id, &server_ids)
                .await
                .map_err(|e| tracing::error!("[Redis] Failed to set user online: {}", e))
                .ok();
        }
    );

    let recv_task = async {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.into_text() {
                handle_event(&text, state.clone(), user.clone(), tx.clone()).await;
            }
        }
    };

    let send_task = async {
        while let Some(json) = rx.recv().await {
            if sender.send(Message::Text(json.as_ref().into())).await.is_err() {
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
    text: &str,
    state: AppState,
    user: User,
    sender: UnboundedSender<Arc<str>>,
) {
    dispatch_event!(text, state, user, sender, [ClientEvent,]);
}

async fn cleanup_user(
    user_id: UserId,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::join!(
        async { state.users.write().await.remove(&user_id) },
        async {
            // Очищаем только каналы, в которых реально состоял пользователь (O(k) вместо O(N всех каналов системы))
            let joined = state.user_channels.write().await.remove(&user_id);
            if let Some(chans) = joined {
                let mut channels = state.channels.write().await;
                for cid in chans {
                    if let Some(users) = channels.get_mut(&cid) {
                        users.remove(&user_id);
                    }
                }
            }
        },
        async {
            let server_ids: Vec<ServerId> = state
                .servers_service
                .retrieve_user_servers(user_id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.id)
                .collect();

            state
                .redis
                .del_user_online(user_id, &server_ids)
                .await
                .map_err(|e| tracing::error!("[Redis] Failed to delete user online: {}", e))
                .ok();
        }
    );
    tracing::info!("[WS] User {} cleaned up", user_id.0);
    Ok(())
}
