use tokio::sync::mpsc::UnboundedSender;

use crate::{events::ServerEvent, state::AppState};
use gilvave_core::dto::user::User;

#[async_trait::async_trait]
pub trait EventHandler {
    async fn handle(self, state: AppState, user: User, sender: UnboundedSender<ServerEvent>);
}

/// Макрос раскрывается в цепочку if let ... else if let ...
/// проверяющие правильность десериализации типов реализующих EventHandler
/// ```
/// dispatch_event!(&raw_text, state, sender, [
///   ClientEvent,
///   AdminEvent,
/// ]);
///
/// // Станет так:
/// if let Ok(event) = serde_json::from_str::<ClientEvent>(&raw_text) {
///    event.handle(state, sender).await;
///    return;
///} else if let Ok(event) = serde_json::from_str::<AdminEvent>(&raw_text) {
///    event.handle(state, sender).await;
///    return;
///}
/// ```
#[macro_export]
macro_rules! dispatch_event {
    ($text:expr, $state:expr, $user:expr, $sender:expr, [$($ty:ty),+ $(,)?]) => {
        $(
            if let Ok(event) = serde_json::from_str::<$ty>($text) {
                event.handle($state, $user, $sender).await;
                return;
            }
        )+
        tracing::error!("[WS] Unknown event type received");
    };
}
