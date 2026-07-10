use azalea::Client;

use crate::ai::client::NimClient;
use crate::ai::context::AiContextBuilder;
use crate::ai::tools::available_tools;
use crate::bot::handler::BOT_CLIENT;
use crate::config::AppConfig;
use crate::executor::actions::run_tool_call;

/// Process a player chat message through NIM with tool support.
pub async fn handle_chat(bot: &Client, sender: &str, message: &str) {
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
        bot.chat("[MineMate] No API key configured — go to Config panel and set one.");
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

    let messages = AiContextBuilder::new(bot_status)
        .with_sender(sender.to_string())
        .with_player_message(message.to_string())
        .build_messages();

    let tools = available_tools();

    let response = match nim.chat(&messages, Some(&tools)).await {
        Ok(r) => r,
        Err(e) => {
            let err = format!("AI API error: {}", e);
            tracing::error!("{}", err);
            bot.chat(&format!("[MineMate] {}", err));
            return;
        }
    };

    let choice = match response.choices.first() {
        Some(c) => c,
        None => return,
    };

    // Handle tool calls
    if let Some(tool_calls) = &choice.message.tool_calls {
        for tool_call in tool_calls {
            tracing::info!("AI requested tool: {}", tool_call.function.name);
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
    if let Some(content) = &choice.message.content {
        let reply = content.trim();
        if !reply.is_empty() {
            bot.chat(reply);
            tracing::info!("AI replied to {}: {}", sender, reply);
        }
    }
}
