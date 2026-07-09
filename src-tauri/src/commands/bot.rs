use crate::bot::events::BotStatus;
use crate::config::AppConfig;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn start_bot(server: String, username: String) -> Result<(), String> {
    tracing::info!("Starting bot for {} on {}", username, server);
    Ok(())
}

#[tauri::command]
pub async fn stop_bot() -> Result<(), String> {
    tracing::info!("Stopping bot");
    Ok(())
}

#[tauri::command]
pub async fn get_bot_status() -> Result<BotStatus, String> {
    Ok(BotStatus::default())
}

#[tauri::command]
pub async fn send_chat(message: String) -> Result<(), String> {
    tracing::info!("Sending chat: {}", message);
    Ok(())
}
