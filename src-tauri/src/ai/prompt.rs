use crate::ai::client::ChatMessage;

pub fn build_context(
    health: f32,
    food: f32,
    x: f64,
    y: f64,
    z: f64,
    inventory: &[String],
    nearby_players: &[String],
    recent_chat: &str,
) -> String {
    let inventory_str = if inventory.is_empty() {
        "Empty".to_string()
    } else {
        inventory.join(", ")
    };

    let players_str = if nearby_players.is_empty() {
        "None".to_string()
    } else {
        nearby_players.join(", ")
    };

    format!(
        r#"You are MineMate, an intelligent Minecraft assistant bot.

Current State:
- Health: {}/20
- Food: {}/20
- Position: x={}, y={}, z={}
- Time: Check world time

Inventory: {}

Nearby Players: {}

Recent Chat: {}

Available Tools: move_to, follow, mine, craft, attack, place_block, build_structure, reply, execute_command, scan_area, give_item, teleport, sort_chests, protect_player

Respond with a JSON tool call to help the player. If no tool is needed, respond with a helpful chat message."#,
        health, food, x, y, z, inventory_str, players_str, recent_chat
    )
}

pub fn create_system_message() -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content: "You are MineMate, a helpful Minecraft bot assistant. You can perform actions using tools. Always respond with either a tool call or a helpful chat message. Be concise and friendly.".to_string(),
    }
}
