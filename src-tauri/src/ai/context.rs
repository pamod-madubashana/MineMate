use crate::ai::client::ChatMessage;
use crate::bot::events::BotStatus;

pub struct AiContextBuilder {
    status: BotStatus,
    inventory: Vec<String>,
    nearby_players: Vec<String>,
    recent_chat: Vec<String>,
}

impl AiContextBuilder {
    pub fn new(status: BotStatus) -> Self {
        Self {
            status,
            inventory: Vec::new(),
            nearby_players: Vec::new(),
            recent_chat: Vec::new(),
        }
    }

    pub fn with_inventory(mut self, inventory: Vec<String>) -> Self {
        self.inventory = inventory;
        self
    }

    pub fn with_nearby_players(mut self, players: Vec<String>) -> Self {
        self.nearby_players = players;
        self
    }

    pub fn with_recent_chat(mut self, chat: Vec<String>) -> Self {
        self.recent_chat = chat;
        self
    }

    pub fn build_system_message(&self) -> ChatMessage {
        ChatMessage {
            role: "system".to_string(),
            content: "You are MineMate, an intelligent Minecraft assistant bot. You help players with building, mining, farming, combat, and general Minecraft questions. You can perform actions using tools. Always respond with either a tool call or a helpful chat message. Be concise and friendly. Never execute arbitrary server commands without explicit player request.".to_string(),
        }
    }

    pub fn build_context_message(&self) -> ChatMessage {
        let inventory_str = if self.inventory.is_empty() {
            "Empty".to_string()
        } else {
            self.inventory.join(", ")
        };

        let players_str = if self.nearby_players.is_empty() {
            "None".to_string()
        } else {
            self.nearby_players.join(", ")
        };

        let chat_str = if self.recent_chat.is_empty() {
            "None".to_string()
        } else {
            self.recent_chat.last().unwrap_or(&String::new()).clone()
        };

        ChatMessage {
            role: "user".to_string(),
            content: format!(
                r#"Current State:
- Health: {}/20
- Food: {}/20
- Position: x={}, y={}, z={}

Inventory: {}

Nearby Players: {}

Recent Chat: {}

What should I do? Respond with a tool call or helpful message."#,
                self.status.health,
                self.status.food,
                self.status.x as i32,
                self.status.y as i32,
                self.status.z as i32,
                inventory_str,
                players_str,
                chat_str
            ),
        }
    }

    pub fn build_messages(&self) -> Vec<ChatMessage> {
        vec![
            self.build_system_message(),
            self.build_context_message(),
        ]
    }
}
