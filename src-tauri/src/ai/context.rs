use crate::ai::client::ChatMessage;
use crate::bot::events::BotStatus;

pub struct AiContextBuilder {
    status: BotStatus,
    inventory: Vec<String>,
    nearby_players: Vec<String>,
    recent_chat: Vec<String>,
    sender: Option<String>,
    player_message: Option<String>,
}

impl AiContextBuilder {
    pub fn new(status: BotStatus) -> Self {
        Self {
            status,
            inventory: Vec::new(),
            nearby_players: Vec::new(),
            recent_chat: Vec::new(),
            sender: None,
            player_message: None,
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

    pub fn with_sender(mut self, sender: String) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn with_player_message(mut self, message: String) -> Self {
        self.player_message = Some(message);
        self
    }

    fn build_system_message(&self) -> ChatMessage {
        ChatMessage {
            role: "system".to_string(),
            content: format!(
                "You are MineMate, an intelligent Minecraft assistant bot on this server. \
                 You help players with building, mining, farming, combat, and general questions. \
                 You can perform actions using tools: move_to, follow, mine, craft, attack, \
                 place_block, reply, execute_command, give_item, teleport, protect_player, and more. \
                 When a player asks you to do something, use the appropriate tool. \
                 If you just need to respond in chat, use the reply tool. \
                 Be concise and friendly. Your master is {}.",
                self.status.master.as_deref().unwrap_or("everyone")
            ),
        }
    }

    fn build_context_message(&self) -> ChatMessage {
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

        let message = self.player_message.as_deref().unwrap_or("No message");
        let sender = self.sender.as_deref().unwrap_or("someone");

        ChatMessage {
            role: "user".to_string(),
            content: format!(
                r#"State:
- Health: {:.0}/20
- Food: {:.0}/20
- Position: ({:.0}, {:.0}, {:.0})
Inventory: {}
Players nearby: {}
Guarding: {}

Player {} says: {}
Respond using a tool call or a brief chat reply."#,
                self.status.health,
                self.status.food,
                self.status.x, self.status.y, self.status.z,
                inventory_str,
                players_str,
                if self.status.guarding { "Yes" } else { "No" },
                sender,
                message,
            ),
        }
    }

    pub fn build_messages(&self) -> Vec<ChatMessage> {
        vec![self.build_system_message(), self.build_context_message()]
    }
}
