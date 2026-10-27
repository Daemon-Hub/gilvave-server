use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::state::AppState;
use gilvave_core::dto::users::User;

#[async_trait::async_trait]
pub trait EventHandler {
    async fn handle(self, state: AppState, user: User, sender: UnboundedSender<Arc<str>>);
}

/// Макрос раскрывается в цепочку if let ... else if let ...
/// проверяющие правильность десериализации типов реализующих EventHandler
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
