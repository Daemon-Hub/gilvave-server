use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedSender};

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

    let hello = ServerEvent::Hello;

    if let Err(e) = sender
        .send(Message::Text(serde_json::to_string(&hello).unwrap().into()))
        .await
    {
        tracing::error!("[WS] Failed to send Hello: {}", e);
        return;
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

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
            state
                .redis
                .set_user_online(
                    user.id,
                    state
                        .server_service
                        .retrieve_user_servers(user.id)
                        .await
                        .unwrap_or(vec![])
                        .into_iter()
                        .map(|s| s.id)
                        .collect(),
                )
                .await
                .map_err(|e| tracing::error!("[Redis] Failed to set user online: {}", e))
                .ok();
        }
    );

    let recv_task = async {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.into_text() {
                handle_event(text.to_string(), state.clone(), user.clone(), tx.clone()).await;
            }
        }
    };

    let send_task = async {
        while let Some(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
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
    sender: UnboundedSender<ServerEvent>,
) {
    dispatch_event!(&text, state, user, sender, [ClientEvent,]);
}

async fn cleanup_user(
    user_id: UserId,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::join!(
        async { state.users.write().await.remove(&user_id) },
        async {
            state
                .channels
                .write()
                .await
                .iter_mut()
                .for_each(|(_, users)| {
                    users.remove(&user_id);
                });
        },
        async {
            state
                .redis
                .del_user_online(
                    user_id,
                    state
                        .server_service
                        .retrieve_user_servers(user_id)
                        .await
                        .unwrap_or(vec![])
                        .into_iter()
                        .map(|s| s.id)
                        .collect(),
                )
                .await
                .map_err(|e| tracing::error!("[Redis] Failed to set user online: {}", e))
                .ok();
        }
    );
    tracing::info!("[WS] User {} cleaned up", user_id.0);
    Ok(())
}
