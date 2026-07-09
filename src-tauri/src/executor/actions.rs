use crate::ai::client::ToolCall;
use serde_json::Value;

pub struct ToolExecutor;

impl ToolExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult, Box<dyn std::error::Error>> {
        let args: Value = serde_json::from_str(&tool_call.function.arguments)?;

        match tool_call.function.name.as_str() {
            "move_to" => {
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let z = args["z"].as_i64().unwrap_or(0) as i32;
                Ok(ToolResult {
                    action: ToolAction::MoveTo { x, y, z },
                    message: format!("Moving to ({}, {}, {})", x, y, z),
                })
            }
            "follow" => {
                let player = args["player"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::Follow { player: player.clone() },
                    message: format!("Following {}", player),
                })
            }
            "mine" => {
                let block = args["block"].as_str().unwrap_or("stone").to_string();
                let count = args["count"].as_u64().unwrap_or(1) as u32;
                Ok(ToolResult {
                    action: ToolAction::Mine { block: block.clone(), count },
                    message: format!("Mining {}x {}", count, block),
                })
            }
            "craft" => {
                let item = args["item"].as_str().unwrap_or("").to_string();
                let count = args["count"].as_u64().unwrap_or(1) as u32;
                Ok(ToolResult {
                    action: ToolAction::Craft { item: item.clone(), count },
                    message: format!("Crafting {}x {}", count, item),
                })
            }
            "attack" => {
                Ok(ToolResult {
                    action: ToolAction::Attack,
                    message: "Attacking nearest hostile".to_string(),
                })
            }
            "place_block" => {
                let block = args["block"].as_str().unwrap_or("stone").to_string();
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let z = args["z"].as_i64().unwrap_or(0) as i32;
                Ok(ToolResult {
                    action: ToolAction::PlaceBlock { block: block.clone(), x, y, z },
                    message: format!("Placing {} at ({}, {}, {})", block, x, y, z),
                })
            }
            "build_structure" => {
                let structure = args["structure"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::BuildStructure { structure: structure.clone() },
                    message: format!("Building {}", structure),
                })
            }
            "reply" => {
                let message = args["message"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::Reply { message: message.clone() },
                    message: format!("Replying: {}", message),
                })
            }
            "execute_command" => {
                let command = args["command"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::ExecuteCommand { command: command.clone() },
                    message: format!("Executing: /{}", command),
                })
            }
            "scan_area" => {
                let radius = args["radius"].as_u64().unwrap_or(10) as u32;
                Ok(ToolResult {
                    action: ToolAction::ScanArea { radius },
                    message: format!("Scanning radius {}", radius),
                })
            }
            "give_item" => {
                let player = args["player"].as_str().unwrap_or("").to_string();
                let item = args["item"].as_str().unwrap_or("").to_string();
                let count = args["count"].as_u64().unwrap_or(1) as u32;
                Ok(ToolResult {
                    action: ToolAction::GiveItem { player: player.clone(), item: item.clone(), count },
                    message: format!("Giving {}x {} to {}", count, item, player),
                })
            }
            "teleport" => {
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let z = args["z"].as_i64().unwrap_or(0) as i32;
                Ok(ToolResult {
                    action: ToolAction::Teleport { x, y, z },
                    message: format!("Teleporting to ({}, {}, {})", x, y, z),
                })
            }
            "sort_chests" => {
                Ok(ToolResult {
                    action: ToolAction::SortChests,
                    message: "Sorting storage chests".to_string(),
                })
            }
            "protect_player" => {
                let player = args["player"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::ProtectPlayer { player: player.clone() },
                    message: format!("Protecting {}", player),
                })
            }
            _ => Err(format!("Unknown tool: {}", tool_call.function.name).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub action: ToolAction,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ToolAction {
    MoveTo { x: i32, y: i32, z: i32 },
    Follow { player: String },
    Mine { block: String, count: u32 },
    Craft { item: String, count: u32 },
    Attack,
    PlaceBlock { block: String, x: i32, y: i32, z: i32 },
    BuildStructure { structure: String },
    Reply { message: String },
    ExecuteCommand { command: String },
    ScanArea { radius: u32 },
    GiveItem { player: String, item: String, count: u32 },
    Teleport { x: i32, y: i32, z: i32 },
    SortChests,
    ProtectPlayer { player: String },
}
