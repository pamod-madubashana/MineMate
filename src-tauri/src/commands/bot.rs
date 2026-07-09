use crate::bot::events::BotStatus;
use crate::bot::handler::BOT_CLIENT;
use crate::bot::handler::connect_to_server;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn start_bot(server: String, username: String) -> Result<(), String> {
    tracing::info!("Starting bot for {} on {}", username, server);

    connect_to_server(&server, &username).await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn stop_bot() -> Result<(), String> {
    tracing::info!("Stopping bot");

    let bot = BOT_CLIENT.read();
    if let Some(client) = bot.as_ref() {
        client.set_connected(false);
        client.emit_event(crate::bot::events::BotEvent::BotStopped);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_bot_status() -> Result<BotStatus, String> {
    let bot = BOT_CLIENT.read();
    match bot.as_ref() {
        Some(client) => Ok(client.get_status()),
        None => Ok(BotStatus::default()),
    }
}

#[tauri::command]
pub async fn send_chat(message: String) -> Result<(), String> {
    tracing::info!("Sending chat: {}", message);

    let bot = BOT_CLIENT.read();
    if let Some(client) = bot.as_ref() {
        client.emit_event(crate::bot::events::BotEvent::ChatMessage {
            player: "Bot".to_string(),
            message: message.clone(),
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn get_connection_status() -> Result<bool, String> {
    let bot = BOT_CLIENT.read();
    match bot.as_ref() {
        Some(client) => Ok(client.is_connected()),
        None => Ok(false),
    }
}
