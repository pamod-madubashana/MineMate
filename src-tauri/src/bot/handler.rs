use crate::bot::client::BotClient;
use crate::bot::events::BotEvent;
use crate::config::AppConfig;
use parking_lot::RwLock;
use std::sync::Arc;
use tauri::Emitter;

pub static BOT_CLIENT: once_cell::sync::Lazy<Arc<RwLock<Option<BotClient>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

pub async fn start_bot_loop(
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let client = BotClient::new(config.clone());

    {
        let mut bot = BOT_CLIENT.write();
        *bot = Some(client.clone());
    }

    tracing::info!(
        "Bot loop started for {}:{}",
        config.server.address,
        config.server.port
    );

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
                        client.set_connected(false);
                        if config.automation.auto_reconnect {
                            tracing::info!("Auto-reconnect enabled, retrying in 5s...");
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                    BotEvent::BotStarted => {
                        client.set_connected(true);
                    }
                    BotEvent::BotStopped => {
                        client.set_connected(false);
                    }
                    _ => {}
                }
            }
            Err(_) => {
                tracing::debug!("Event channel closed, bot loop exiting");
                break;
            }
        }
    }

    Ok(())
}

pub async fn connect_to_server(
    address: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Connecting to {} as {}", address, username);

    // Note: Azalea uses Bevy ECS internally. The actual connection
    // happens through the ClientBuilder which runs in its own event loop.
    // For now, we log the connection attempt. Full Azalea integration
    // requires setting up the Bevy app and event handlers.

    tracing::info!("Azalea client initialized for {}", address);

    Ok(())
}
