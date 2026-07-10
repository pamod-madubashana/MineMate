use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use azalea::Client;
use parking_lot::RwLock;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

use crate::bot::events::{BotEvent, BotStatus};
use crate::config::AppConfig;

#[derive(Clone)]
pub struct BotClient {
    pub status: Arc<RwLock<BotStatus>>,
    pub event_tx: broadcast::Sender<BotEvent>,
    pub config: Arc<RwLock<AppConfig>>,
    pub connected: Arc<RwLock<bool>>,
    pub app_handle: AppHandle,
    pub azalea_client: Arc<RwLock<Option<Client>>>,
    pub follow_stop: Arc<AtomicBool>,
    pub guarding: Arc<AtomicBool>,
    pub master: Arc<RwLock<Option<String>>>,
    pub start_time: Arc<RwLock<Option<std::time::Instant>>>,
}

impl BotClient {
    pub fn new(config: AppConfig, app_handle: AppHandle) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            status: Arc::new(RwLock::new(BotStatus::default())),
            event_tx,
            config: Arc::new(RwLock::new(config)),
            connected: Arc::new(RwLock::new(false)),
            app_handle,
            azalea_client: Arc::new(RwLock::new(None)),
            follow_stop: Arc::new(AtomicBool::new(false)),
            guarding: Arc::new(AtomicBool::new(false)),
            master: Arc::new(RwLock::new(None)),
            start_time: Arc::new(RwLock::new(None)),
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
        let _ = self.event_tx.send(event.clone());
        let _ = self.app_handle.emit("bot://event", event);
    }

    pub fn set_connected(&self, connected: bool) {
        *self.connected.write() = connected;
    }

    pub fn update_status(&self, status: BotStatus) {
        *self.status.write() = status;
    }

    pub fn set_guarding(&self, enabled: bool) {
        self.guarding.store(enabled, std::sync::atomic::Ordering::Relaxed);
        {
            let mut status = self.status.write();
            status.guarding = enabled;
        }
        self.emit_event(BotEvent::GuardModeChanged { enabled });
    }

    pub fn is_guarding(&self) -> bool {
        self.guarding.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_master(&self, master: Option<String>) {
        let m = master.clone();
        *self.master.write() = master;
        {
            let mut status = self.status.write();
            status.master = m;
        }
    }

    pub fn get_master(&self) -> Option<String> {
        self.master.read().clone()
    }

    pub fn set_following(&self, player: Option<String>) {
        let mut status = self.status.write();
        status.following = player;
    }
}
