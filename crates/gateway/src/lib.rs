mod handler;
pub mod ws;

use axum::Router;
use std::sync::Arc;

use gilvave_realtime::Realtime;

pub fn router(rt: Arc<Realtime>) -> Router {
    Router::new().route(
        "/ws",
        axum::routing::get(move |ws| crate::ws::ws_handler(ws, rt.clone())),
    )
}
