use azalea::Client;

use crate::ai::client::NimClient;
use crate::ai::context::AiContextBuilder;
use crate::ai::tools::available_tools;
use crate::bot::handler::BOT_CLIENT;
use crate::config::AppConfig;
use crate::executor::actions::run_tool_call;

/// Process a player chat message through NIM with tool support.
///
/// Builds context from bot state, includes available tools,
/// and executes any tool calls the AI decides to make.
/// Falls back to text reply if no tool is called.
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

    // Extract response or error as a Send-friendly type before any .await
    let response = match nim.chat(&messages, Some(&tools)).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("AI chat error: {}", e);
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
