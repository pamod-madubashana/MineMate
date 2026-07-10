use std::collections::VecDeque;
use std::sync::Mutex;

use azalea::ecs::query::{With, Without};
use azalea::entity::metadata::Player;
use azalea::Client;
use once_cell::sync::Lazy;

use crate::ai::client::NimClient;
use crate::ai::context::AiContextBuilder;
use crate::ai::tools::available_tools;
use crate::bot::handler::BOT_CLIENT;
use crate::config::AppConfig;
use crate::executor::actions::run_tool_call;

static RECENT_CHAT: Lazy<Mutex<VecDeque<String>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(20)));

/// Process a player chat message through NIM with tool support.
pub async fn handle_chat(bot: &Client, sender: &str, message: &str) {
    // Store in recent chat ring buffer
    {
        let mut chat = RECENT_CHAT.lock().unwrap();
        if chat.len() >= 20 {
            chat.pop_front();
        }
        chat.push_back(format!("<{}> {}", sender, message));
    }

    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            let err = format!("Config load failed: {}", e);
            tracing::error!("{}", err);
            bot.chat(&format!("[MineMate Error] {}", err));
            return;
        }
    };

    if config.ai.api_key.is_empty() {
        bot.chat("No API key configured — go to Config panel and set one.");
        return;
    }

    let bot_username = bot.username();
    if sender == bot_username {
        return;
    }

    let nim = NimClient::new(config.ai.api_key.clone(), config.ai.model.clone());

    let bot_status = BOT_CLIENT
        .read()
        .as_ref()
        .map(|c| c.get_status())
        .unwrap_or_default();

    // Gather nearby player names
    let nearby_players: Vec<String> = bot
        .nearest_entities::<(With<Player>, Without<azalea::entity::LocalEntity>)>()
        .ok()
        .map(|entities| {
            entities
                .iter()
                .filter_map(|e| {
                    let name = e
                        .get_component::<azalea::player::GameProfileComponent>()?;
                    Some(name.name.clone())
                })
                .collect()
        })
        .unwrap_or_default();

    // Gather inventory items (first 36 slots = hotbar + main inventory)
    let inventory: Vec<String> = bot
        .get_inventory()
        .ok()
        .and_then(|inv| inv.slots())
        .map(|stacks| {
            stacks
                .iter()
                .take(36)
                .filter(|s| !s.is_empty())
                .map(|s| {
                    let item_name = format!("{:?}", s.kind());
                    let count = s.count();
                    if count > 1 {
                        format!("{}x{}", item_name, count)
                    } else {
                        item_name
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Get recent chat
    let recent_chat: Vec<String> = {
        let chat = RECENT_CHAT.lock().unwrap();
        chat.iter().cloned().collect()
    };

    let messages = AiContextBuilder::new(bot_status)
        .with_inventory(inventory)
        .with_nearby_players(nearby_players)
        .with_recent_chat(recent_chat)
        .with_sender(sender.to_string())
        .with_player_message(message.to_string())
        .build_messages();

    let tools = available_tools();

    let response = match nim.chat(&messages, Some(&tools)).await {
        Ok(r) => r,
        Err(e) => {
            let err = format!("AI API error: {}", e);
            tracing::error!("{}", err);
            bot.chat(&format!("{}", err));
            return;
        }
    };

    // Handle tool calls
    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            tracing::info!("AI requested tool: {}", tool_call.name);
            match run_tool_call(tool_call).await {
                Ok(Some(reply)) => {
                    bot.chat(&reply);
                }
                Ok(None) => {}
                Err(e) => {
                    bot.chat(&format!("Sorry, I couldn't do that: {}", e));
                }
            }
        }
        return;
    }

    // Fall back to text reply
    if let Some(content) = &response.content {
        let reply = content.trim();
        if !reply.is_empty() {
            bot.chat(reply);
            tracing::info!("AI replied to {}: {}", sender, reply);
        }
    }
}
