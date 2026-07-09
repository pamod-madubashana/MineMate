use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotEvent {
    ChatMessage { player: String, message: String },
    PlayerJoined { name: String },
    PlayerLeft { name: String },
    Disconnected { reason: String },
    HealthChanged { health: f32, food: f32 },
    InventoryChanged,
    PositionChanged { x: f64, y: f64, z: f64 },
    BotStarted,
    BotStopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStatus {
    pub connected: bool,
    pub server: String,
    pub username: String,
    pub health: f32,
    pub food: f32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub uptime_seconds: u64,
}

impl Default for BotStatus {
    fn default() -> Self {
        Self {
            connected: false,
            server: String::new(),
            username: String::new(),
            health: 20.0,
            food: 20.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            uptime_seconds: 0,
        }
    }
}
