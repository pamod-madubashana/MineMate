use std::net::ToSocketAddrs;

use azalea::player::GameProfileComponent;
use azalea::prelude::*;
use azalea::protocol::address::{ResolvedAddr, ServerAddr};

use crate::bot::client::BotClient;
use crate::bot::events::BotEvent;
use crate::bot::events::BotStatus;
use crate::bot::handler::{BOT_CLIENT, BOT_RUNNING};
use crate::config::AppConfig;
use crate::executor::audit::get_audit_logger;
use crate::executor::security::SecurityValidator;

async fn azalea_handler(bot: Client, event: Event, _state: azalea::NoState) {
    match event {
        azalea::Event::Spawn => {
            tracing::info!("Bot spawned in world");
            if let Some(b) = BOT_CLIENT.read().as_ref() {
                b.azalea_client.write().replace(bot.clone());
                b.set_connected(true);
                b.start_time.write().replace(std::time::Instant::now());
                b.emit_event(BotEvent::BotStarted);

                // Start the guard background loop
                let guard_bot = bot.clone();
                let guard_flag = b.guarding.clone();
                let master = b.master.clone();
                crate::bot::guard::start_guard_loop(guard_bot, guard_flag, master);

                // Default: find nearest player and follow them
                let follow_bot = bot.clone();
                let client = b.clone();
                tokio::task::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    let players = match follow_bot.nearby_players() {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::error!("Failed to get nearby players: {}", e);
                            return;
                        }
                    };
                    let nearest = players.first().cloned();
                    if let Some(player) = nearest {
                        let name = match player
                            .component::<GameProfileComponent>()
                            .ok()
                            .map(|g| g.0.name.clone())
                        {
                            Some(n) if !n.is_empty() => n,
                            _ => return,
                        };
                        tracing::info!("Default follow: nearest player is {}", name);
                        client.set_master(Some(name.clone()));
                        client.set_guarding(true);
                        client.follow_stop.store(false, std::sync::atomic::Ordering::Relaxed);
                        client.set_following(Some(name.clone()));
                        crate::bot::follow::start_following(follow_bot.clone(), name, client.follow_stop.clone());
                        follow_bot.chat("I will protect you!");
                    }
                });
            }
        }
        azalea::Event::Disconnect(reason) => {
            tracing::warn!("Bot disconnected: {:?}", reason);
            if let Some(b) = BOT_CLIENT.read().as_ref() {
                b.azalea_client.write().take();
                b.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
                b.set_connected(false);
                b.emit_event(BotEvent::Disconnected {
                    reason: format!("{:?}", reason),
                });
            }
        }
        azalea::Event::Chat(chat) => {
            let sender = chat.sender().unwrap_or_default();
            let content = chat.content();
            tracing::debug!("Chat event — sender: {:?} content: {:?}", sender, content);
            if let Some(b) = BOT_CLIENT.read().as_ref() {
                b.emit_event(BotEvent::ChatMessage {
                    player: sender.clone(),
                    message: content.clone(),
                });
            }
            // AI auto-reply in a background task
            if !sender.is_empty() && !content.is_empty() {
                if let Some(b) = BOT_CLIENT.read().as_ref() {
                    if let Some(azalea) = b.azalea_client.read().as_ref() {
                        let bot = azalea.clone();
                        let s = sender.clone();
                        let m = content.clone();
                        tokio::task::spawn(async move {
                            crate::ai::chat_handler::handle_chat(&bot, &s, &m).await;
                        });
                    }
                }
            }
        }
        azalea::Event::Death(reason) => {
            let msg = reason
                .as_ref()
                .map(|r| r.message.to_string())
                .unwrap_or_default();
            tracing::warn!("Bot died: {}", msg);
            if let Some(b) = BOT_CLIENT.read().as_ref() {
                b.emit_event(BotEvent::Disconnected {
                    reason: format!("Bot died: {}", msg),
                });
            }
        }
        azalea::Event::Tick => {
            if !*BOT_RUNNING.read() {
                tracing::info!("Bot stop requested, exiting...");
                bot.exit();
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub async fn start_bot(server: String, username: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("Starting bot for {} on {}", username, server);

    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let _validator = SecurityValidator::new(config.bot.permission_mode.clone());

    get_audit_logger().log_success(
        "start_bot",
        Some(&username),
        &format!("Connecting to {}", server),
    );

    // Create BotClient and store it so events can be emitted
    let client = BotClient::new(config.clone(), app_handle.clone());
    {
        let mut bot = BOT_CLIENT.write();
        *bot = Some(client.clone());
    }

    {
        let mut running = BOT_RUNNING.write();
        *running = true;
    }

    let address = server.clone();
    let uname = username.clone();

    // Spawn Azalea connection in a blocking task since it runs Bevy ECS
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            tracing::info!("Connecting to {} as {}", address, uname);

            let account = Account::offline(&uname);

            // Pre-resolve to IPv4 — Azalea's built-in hickory-resolver picks
            // the first DNS result, which can be an unreachable NAT64 IPv6
            // address on some networks (e.g. Aternos servers).
            let resolved = match resolve_ipv4(&address) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to resolve {} to IPv4: {}", address, e);
                    if let Some(b) = BOT_CLIENT.read().as_ref() {
                        b.set_connected(false);
                        let _ = b.event_tx.send(BotEvent::Disconnected {
                            reason: format!("DNS resolution failed: {}", e),
                        });
                    }
                    return;
                }
            };

            let exit = ClientBuilder::new()
                .set_handler(azalea_handler)
                .start(account, &resolved)
                .await;

            tracing::info!("Bot exited with: {:?}", exit);

            if let Some(b) = BOT_CLIENT.read().as_ref() {
                b.azalea_client.write().take();
                b.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
                b.set_connected(false);
                b.emit_event(BotEvent::BotStopped);
            }
        });
    });

    Ok(())
 }

/// Resolve a Minecraft server address to an IPv4 address only.
///
/// The standard library DNS resolver is used instead of Azalea's
/// hickory-resolver, which may return an IPv6 NAT64 address first
/// that is unreachable on the local network.
fn resolve_ipv4(addr: &str) -> Result<ResolvedAddr, String> {
    let server = ServerAddr::try_from(addr).map_err(|_| format!("Invalid address: {}", addr))?;
    let ips: Vec<std::net::SocketAddr> = addr
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed: {}", e))?
        .collect();
    let ipv4 = ips
        .into_iter()
        .find(|a| a.is_ipv4())
        .ok_or_else(|| format!("No IPv4 address found for {}", addr))?;
    Ok(ResolvedAddr {
        server,
        socket: ipv4,
    })
}

#[tauri::command]
pub async fn stop_bot() -> Result<(), String> {
    tracing::info!("Stopping bot");

    {
        let mut running = BOT_RUNNING.write();
        *running = false;
    }

    {
        let mut bot = BOT_CLIENT.write();
        if let Some(client) = bot.as_ref() {
            client.azalea_client.write().take();
            client.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
            client.set_connected(false);
            client.emit_event(BotEvent::BotStopped);
        }
        *bot = None;
    }

    get_audit_logger().log_success("stop_bot", None, "Bot stopped");

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
    let client = bot.as_ref().ok_or_else(|| "Bot not started".to_string())?;
    let azalea = client.azalea_client.read();
    let azalea = azalea.as_ref().ok_or_else(|| "Bot not yet connected to server".to_string())?;
    azalea.chat(&message);

    client.emit_event(BotEvent::ChatMessage {
        player: "Bot".to_string(),
        message: message.clone(),
    });

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

#[tauri::command]
pub async fn follow_player(player: String) -> Result<(), String> {
    tracing::info!("Follow player: {}", player);

    let bot = BOT_CLIENT.read();
    let client = bot.as_ref().ok_or_else(|| "Bot not started".to_string())?;
    let azalea = client.azalea_client.read();
    let azalea = azalea.as_ref().ok_or_else(|| "Bot not yet connected".to_string())?;

    client.follow_stop.store(false, std::sync::atomic::Ordering::Relaxed);
    crate::bot::follow::start_following(azalea.clone(), player.clone(), client.follow_stop.clone());

    get_audit_logger().log_success("follow_player", Some(&player), "Started following");
    Ok(())
}

#[tauri::command]
pub async fn stop_following() -> Result<(), String> {
    tracing::info!("Stop following");

    let bot = BOT_CLIENT.read();
    if let Some(client) = bot.as_ref() {
        client.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(azalea) = client.azalea_client.read().as_ref() {
            azalea.stop_pathfinding();
        }
    }

    get_audit_logger().log_success("stop_following", None, "Stopped following");
    Ok(())
}

#[tauri::command]
pub async fn set_guard_mode(enabled: bool) -> Result<(), String> {
    tracing::info!("Setting guard mode: {}", enabled);

    let bot = BOT_CLIENT.read();
    let client = bot.as_ref().ok_or_else(|| "Bot not started".to_string())?;
    client.set_guarding(enabled);

    if enabled {
        if let Some(azalea) = client.azalea_client.read().as_ref() {
            azalea.chat("Guard mode activated!");
        }
    }

    get_audit_logger().log_success("set_guard_mode", None, if enabled { "Enabled" } else { "Disabled" });
    Ok(())
}

#[tauri::command]
pub async fn get_guard_status() -> Result<bool, String> {
    let bot = BOT_CLIENT.read();
    match bot.as_ref() {
        Some(client) => Ok(client.is_guarding()),
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn set_master_player(player: String) -> Result<(), String> {
    tracing::info!("Setting master: {}", player);

    let bot = BOT_CLIENT.read();
    let client = bot.as_ref().ok_or_else(|| "Bot not started".to_string())?;
    client.set_master(Some(player.clone()));

    get_audit_logger().log_success("set_master", Some(&player), "Master set");
    Ok(())
}
