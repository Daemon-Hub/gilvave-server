use tokio::sync::broadcast;

use gilvave_core::event::DispatchPayload;

#[derive(Clone)]
pub struct Realtime {
    tx: broadcast::Sender<DispatchPayload>,
}

impl Realtime {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DispatchPayload> {
        self.tx.subscribe()
    }

    pub fn send(&self, event: DispatchPayload) {
        let _ = self.tx.send(event);
    }
}

impl Default for Realtime {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }
}
