use tokio::sync::broadcast;

use gilvave_core::event::DispatchPayload;

#[derive(Clone)]
pub struct Realtime {
    tx: broadcast::Sender<DispatchPayload>,
}

impl Realtime {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DispatchPayload> {
        self.tx.subscribe()
    }

    pub fn send(&self, event: DispatchPayload) {
        let _ = self.tx.send(event);
    }
}
