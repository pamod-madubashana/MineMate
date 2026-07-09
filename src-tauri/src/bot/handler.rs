#![allow(dead_code)]

use crate::bot::client::BotClient;
use crate::bot::events::BotEvent;
use parking_lot::RwLock;
use std::sync::Arc;

pub static BOT_CLIENT: once_cell::sync::Lazy<Arc<RwLock<Option<BotClient>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

pub static BOT_RUNNING: once_cell::sync::Lazy<Arc<RwLock<bool>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(false)));

pub async fn stop_bot() {
    {
        let mut running = BOT_RUNNING.write();
        *running = false;
    }

    {
        let mut bot = BOT_CLIENT.write();
        if let Some(client) = bot.as_ref() {
            client.set_connected(false);
            let _ = client.event_tx.send(BotEvent::BotStopped);
        }
        *bot = None;
    }

    tracing::info!("Bot stopped");
}
