use azalea::Client;

use crate::ai::client::{ChatMessage, NimClient};
use crate::config::AppConfig;

/// Process a player chat message through NIM and send a reply.
///
/// Skips if no API key is configured or if the sender is the bot itself.
pub async fn handle_chat(bot: &Client, sender: &str, message: &str) {
    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to load config for AI reply: {}", e);
            return;
        }
    };

    if config.ai.api_key.is_empty() {
        return;
    }

    let bot_username = bot.username();
    if sender == bot_username {
        return;
    }

    let nim = NimClient::new(config.ai.api_key, config.ai.model);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: format!(
                "You are MineMate, a helpful Minecraft staff bot on this server. \
                 You help players with building, mining, farming, and general questions. \
                 Be concise, friendly, and keep responses to 1-2 short sentences. \
                 The player who messaged you is {}.",
                sender
            ),
        },
        ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        },
    ];

    match nim.chat(&messages, None).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                if let Some(content) = &choice.message.content {
                    let reply = content.trim();
                    if !reply.is_empty() {
                        bot.chat(reply);
                        tracing::info!("AI replied to {}: {}", sender, reply);
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("AI chat error: {}", e);
        }
    }
}
