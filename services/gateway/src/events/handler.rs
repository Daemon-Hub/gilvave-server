use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;

use crate::state::AppState;

#[async_trait::async_trait]
pub trait EventHandler {
    async fn handle(self, state: AppState, sender: &mut SplitSink<WebSocket, Message>);
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
    ($text:expr, $state:expr, $sender:expr, [$($ty:ty),+ $(,)?]) => {
        $(
            if let Ok(event) = serde_json::from_str::<$ty>($text) {
                event.handle($state, $sender).await;
                return;
            }
        )+
        eprintln!("Unknown event type received");
    };
}
