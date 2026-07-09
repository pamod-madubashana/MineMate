use crate::bot::client::BotClient;
use crate::bot::events::{BotEvent, BotStatus};
use crate::config::AppConfig;
use std::sync::Arc;
use parking_lot::RwLock;
use tauri::Manager;

pub static BOT_CLIENT: once_cell::sync::Lazy<Arc<RwLock<Option<BotClient>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

pub async fn start_bot_loop(app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let client = BotClient::new(config.clone());

    {
        let mut bot = BOT_CLIENT.write();
        *bot = Some(client.clone());
    }

    tracing::info!("Bot loop started for {}:{}", config.server.address, config.server.port);

    let mut rx = client.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => {
                let _ = app_handle.emit("bot://event", &event);
                match &event {
                    BotEvent::ChatMessage { player, message } => {
                        tracing::info!("[{}] {}", player, message);
                    }
                    BotEvent::Disconnected { reason } => {
                        tracing::warn!("Disconnected: {}", reason);
                        if config.automation.auto_reconnect {
                            tracing::info!("Auto-reconnect enabled, retrying in 5s...");
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                tracing::debug!("Event channel closed,bot loop exiting");
                break;
            }
        }
    }

    Ok(())
}
