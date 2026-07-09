#![allow(dead_code)]

use crate::bot::events::BotEvent;
use crate::config::AppConfig;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct AutomationEngine {
    config: Arc<RwLock<AppConfig>>,
}

impl AutomationEngine {
    pub fn new(config: Arc<RwLock<AppConfig>>) -> Self {
        Self { config }
    }

    pub fn handle_event(&self, event: &BotEvent) -> Option<BotAction> {
        let config = self.config.read();

        match event {
            BotEvent::ChatMessage { player, message } => self.handle_chat(player, message, &config),
            BotEvent::HealthChanged { health, food } => self.handle_health(*health, *food, &config),
            _ => None,
        }
    }

    fn handle_chat(&self, player: &str, message: &str, config: &AppConfig) -> Option<BotAction> {
        let message_lower = message.to_lowercase();

        if config.automation.welcome_messages && message_lower.contains("joined the game") {
            return Some(BotAction::SendMessage {
                message: format!("Welcome {}! Type 'help' for commands.", player),
            });
        }

        if message_lower == "help" {
            return Some(BotAction::SendMessage {
                message: "Available commands: help, kit, where, follow, stop".to_string(),
            });
        }

        if message_lower == "kit" && config.automation.starter_kit_on_respawn {
            let items: Vec<String> = config
                .starter_kit
                .iter()
                .map(|item| format!("{}x{}", item.item, item.count))
                .collect();
            return Some(BotAction::SendMessage {
                message: format!("Giving starter kit: {}", items.join(", ")),
            });
        }

        None
    }

    fn handle_health(&self, health: f32, food: f32, config: &AppConfig) -> Option<BotAction> {
        if config.automation.auto_eat && food < 10.0 {
            return Some(BotAction::Eat);
        }

        if health < 10.0 {
            return Some(BotAction::SendMessage {
                message: "Warning: Low health!".to_string(),
            });
        }

        None
    }
}

#[derive(Debug, Clone)]
pub enum BotAction {
    SendMessage { message: String },
    Eat,
    Sleep,
    Follow { player: String },
    Guard { player: String },
    Mine { block: String, count: u32 },
    Build { structure: String },
    MoveTo { x: i32, y: i32, z: i32 },
}
