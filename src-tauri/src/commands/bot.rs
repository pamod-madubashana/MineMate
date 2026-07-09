use crate::bot::events::BotStatus;
use crate::bot::handler::connect_to_server;
use crate::bot::handler::BOT_CLIENT;
use crate::config::AppConfig;
use crate::executor::audit::get_audit_logger;
use crate::executor::security::SecurityValidator;

#[tauri::command]
pub async fn start_bot(server: String, username: String) -> Result<(), String> {
    tracing::info!("Starting bot for {} on {}", username, server);

    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let _validator = SecurityValidator::new(config.bot.permission_mode);

    get_audit_logger().log_success(
        "start_bot",
        Some(&username),
        &format!("Connecting to {}", server),
    );

    connect_to_server(&server, &username).await.map_err(|e| {
        get_audit_logger().log_failure("start_bot", Some(&username), &e.to_string());
        format!("Failed to connect: {}", e)
    })?;

    Ok(())
}

#[tauri::command]
pub async fn stop_bot() -> Result<(), String> {
    tracing::info!("Stopping bot");

    get_audit_logger().log_success("stop_bot", None, "Bot stopped");

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
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let validator = SecurityValidator::new(config.bot.permission_mode);

    if let Err(e) = validator.validate_chat_message(&message) {
        get_audit_logger().log_blocked("send_chat", None, &e.to_string());
        return Err(e.to_string());
    }

    tracing::info!("Sending chat: {}", message);

    let bot = BOT_CLIENT.read();
    if let Some(client) = bot.as_ref() {
        client.emit_event(crate::bot::events::BotEvent::ChatMessage {
            player: "Bot".to_string(),
            message: message.clone(),
        });
    }

    get_audit_logger().log_success("send_chat", None, &message);

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

#[tauri::command]
pub async fn get_audit_logs(count: u32) -> Result<Vec<serde_json::Value>, String> {
    let logs = get_audit_logger().get_recent_logs(count as usize);
    let values: Vec<serde_json::Value> = logs
        .iter()
        .map(|log| {
            serde_json::json!({
                "timestamp": log.timestamp,
                "action": log.action,
                "player": log.player,
                "result": format!("{:?}", log.result),
                "details": log.details
            })
        })
        .collect();
    Ok(values)
}
