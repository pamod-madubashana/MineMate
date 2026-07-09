use crate::bot::events::{BotEvent, BotStatus};
use crate::config::AppConfig;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct BotClient {
    pub status: Arc<RwLock<BotStatus>>,
    pub event_tx: broadcast::Sender<BotEvent>,
    pub config: Arc<RwLock<AppConfig>>,
    pub connected: Arc<RwLock<bool>>,
}

impl BotClient {
    pub fn new(config: AppConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            status: Arc::new(RwLock::new(BotStatus::default())),
            event_tx,
            config: Arc::new(RwLock::new(config)),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    pub fn get_status(&self) -> BotStatus {
        self.status.read().clone()
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BotEvent> {
        self.event_tx.subscribe()
    }

    pub fn emit_event(&self, event: BotEvent) {
        let _ = self.event_tx.send(event);
    }

    pub fn set_connected(&self, connected: bool) {
        *self.connected.write() = connected;
    }

    pub fn update_status(&self, status: BotStatus) {
        *self.status.write() = status;
    }
}
