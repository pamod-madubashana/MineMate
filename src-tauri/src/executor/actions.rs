use azalea::ecs::query::With;
use azalea::entity::metadata::AbstractMonster;
use azalea::pathfinder::PathfinderClientExt;
use azalea::registry::builtin::BlockKind;
use azalea::Client;

use crate::ai::client::ToolCall;
use crate::bot::client::BotClient;
use crate::bot::handler::BOT_CLIENT;
use crate::executor::security::SecurityValidator;
use serde_json::Value;

pub struct ToolExecutor;

impl ToolExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult, Box<dyn std::error::Error>> {
        let args: Value = serde_json::from_str(&tool_call.arguments)?;

        match tool_call.name.as_str() {
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
                    action: ToolAction::Follow {
                        player: player.clone(),
                    },
                    message: format!("Following {}", player),
                })
            }
            "mine" => {
                let block = args["block"].as_str().unwrap_or("stone").to_string();
                let count = args["count"].as_u64().unwrap_or(1) as u32;
                Ok(ToolResult {
                    action: ToolAction::Mine {
                        block: block.clone(),
                        count,
                    },
                    message: format!("Mining {}x {}", count, block),
                })
            }
            "craft" => {
                let item = args["item"].as_str().unwrap_or("").to_string();
                let count = args["count"].as_u64().unwrap_or(1) as u32;
                Ok(ToolResult {
                    action: ToolAction::Craft {
                        item: item.clone(),
                        count,
                    },
                    message: format!("Crafting {}x {}", count, item),
                })
            }
            "attack" => Ok(ToolResult {
                action: ToolAction::Attack,
                message: "Attacking nearest hostile".to_string(),
            }),
            "place_block" => {
                let block = args["block"].as_str().unwrap_or("stone").to_string();
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let z = args["z"].as_i64().unwrap_or(0) as i32;
                Ok(ToolResult {
                    action: ToolAction::PlaceBlock {
                        block: block.clone(),
                        x,
                        y,
                        z,
                    },
                    message: format!("Placing {} at ({}, {}, {})", block, x, y, z),
                })
            }
            "build_structure" => {
                let structure = args["structure"].as_str().unwrap_or("").to_string();
                let x = args["x"].as_i64().map(|v| v as i32);
                let y = args["y"].as_i64().map(|v| v as i32);
                let z = args["z"].as_i64().map(|v| v as i32);
                Ok(ToolResult {
                    action: ToolAction::BuildStructure {
                        structure: structure.clone(),
                        x,
                        y,
                        z,
                    },
                    message: format!("Building {}", structure),
                })
            }
            "reply" => {
                let message = args["message"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::Reply {
                        message: message.clone(),
                    },
                    message: format!("Replying: {}", message),
                })
            }
            "execute_command" => {
                let command = args["command"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::ExecuteCommand {
                        command: command.clone(),
                    },
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
                    action: ToolAction::GiveItem {
                        player: player.clone(),
                        item: item.clone(),
                        count,
                    },
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
            "sort_chests" => Ok(ToolResult {
                action: ToolAction::SortChests,
                message: "Sorting storage chests".to_string(),
            }),
            "protect_player" => {
                let player = args["player"].as_str().unwrap_or("").to_string();
                Ok(ToolResult {
                    action: ToolAction::ProtectPlayer {
                        player: player.clone(),
                    },
                    message: format!("Protecting {}", player),
                })
            }
            _ => Err(format!("Unknown tool: {}", tool_call.name).into()),
        }
    }
}

/// Run the full pipeline: parse a tool call, validate, and execute it against the bot.
pub async fn run_tool_call(tool_call: &ToolCall) -> Result<Option<String>, String> {
    let executor = ToolExecutor::new();
    let result = executor.execute(tool_call).map_err(|e| e.to_string())?;

    let config = crate::config::AppConfig::load().map_err(|e| e.to_string())?;
    let validator = SecurityValidator::new(config.bot.permission_mode);
    validator.validate_tool_action(&result.action).map_err(|e| e.to_string())?;

    execute_via_client(&result.action).await
}

/// Execute a parsed ToolAction against the live bot client.
async fn execute_via_client(action: &ToolAction) -> Result<Option<String>, String> {
    let bot_client = get_bot_client().ok_or("Bot not started")?;
    let azalea = get_azalea_client().ok_or("Bot not connected")?;

    match action {
        ToolAction::MoveTo { x, y, z } => {
            let pos = azalea::Vec3::new(*x as f64, *y as f64, *z as f64);
            crate::bot::pathfinding::open_nearby_doors(&azalea, 3).await;
            azalea.start_goto_with_opts(
                azalea::pathfinder::goals::RadiusGoal { pos, radius: 1.0 },
                crate::bot::pathfinding::smart_pathfinder_opts(),
            );
            Ok(Some(format!("Moving to ({}, {}, {})", x, y, z)))
        }
        ToolAction::Follow { player } => {
            bot_client.follow_stop.store(false, std::sync::atomic::Ordering::Relaxed);
            bot_client.set_following(Some(player.clone()));
            crate::bot::follow::start_following(
                azalea.clone(),
                player.clone(),
                bot_client.follow_stop.clone(),
            );
            Ok(Some(format!("Now following {}", player)))
        }
        ToolAction::Attack => {
            match azalea.nearest_entity_by::<(), With<AbstractMonster>>(|_| true) {
                Ok(Some(target)) => {
                    let _ = target.look_at();
                    target.attack();
                    Ok(Some("Attacking nearest hostile".to_string()))
                }
                Ok(None) => {
                    azalea.chat("No hostile entities nearby");
                    Ok(Some("No hostile entities nearby".to_string()))
                }
                Err(e) => {
                    tracing::error!("Failed to query hostile entity: {}", e);
                    Ok(Some(format!("Error: {}", e)))
                }
            }
        }
        ToolAction::Reply { message } => {
            azalea.chat(message);
            Ok(None)
        }
        ToolAction::ExecuteCommand { command } => {
            azalea.chat(&format!("/{}", command));
            Ok(Some(format!("Executed /{}", command)))
        }
        ToolAction::ProtectPlayer { player } => {
            bot_client.set_guarding(true);
            bot_client.set_master(Some(player.clone()));
            azalea.chat(&format!("I will protect you, {}!", player));
            Ok(Some(format!("Now protecting {}", player)))
        }
        ToolAction::Teleport { x, y, z } => {
            azalea.chat(&format!("/tp {} {} {}", x, y, z));
            Ok(Some(format!("Teleported to ({}, {}, {})", x, y, z)))
        }
        ToolAction::GiveItem { player, item, count } => {
            azalea.chat(&format!("/give {} {} {}", player, item, count));
            Ok(Some(format!("Giving {} {} to {}", count, item, player)))
        }
        ToolAction::Mine { block, count: _ } => {
            let block_kind: BlockKind = match block.parse() {
                Ok(k) => k,
                Err(_) => {
                    let msg = format!("Unknown block type '{}'", block);
                    azalea.chat(&format!("{}", msg));
                    return Ok(Some(msg));
                }
            };
            let target_states = azalea::block::BlockStates::from([block_kind]);

            let bot_pos = match azalea.entity().position() {
                Ok(p) => p,
                Err(e) => {
                    let msg = format!("Can't get position: {}", e);
                    azalea.chat(&format!("{}", msg));
                    return Ok(Some(msg));
                }
            };
            let search_origin = azalea::BlockPos::new(
                bot_pos.x as i32, bot_pos.y as i32, bot_pos.z as i32,
            );

            let world = match azalea.world() {
                Ok(w) => w,
                Err(e) => {
                    let msg = format!("World not available: {}", e);
                    azalea.chat(&format!("[MineMate] {}", msg));
                    return Ok(Some(msg));
                }
            };
            let block_pos = {
                let w = world.read();
                w.find_block(search_origin, &target_states)
            };
            let block_pos = match block_pos {
                Some(p) => p,
                None => {
                    let msg = format!("No '{}' found nearby", block);
                    azalea.chat(&format!("[MineMate] {}", msg));
                    return Ok(Some(msg));
                }
            };

            // Walk within reach of the block
            let target = azalea::Vec3::new(
                block_pos.x as f64 + 0.5,
                block_pos.y as f64,
                block_pos.z as f64 + 0.5,
            );
            azalea.goto(azalea::pathfinder::goals::RadiusGoal {
                pos: target,
                radius: 2.0,
            })
            .await;

            // Look at the block and mine it
            azalea.look_at(block_pos.center());
            azalea.mine(block_pos).await;

            let msg = format!("Mined {} at ({}, {}, {})", block, block_pos.x, block_pos.y, block_pos.z);
            Ok(Some(msg))
        }
        ToolAction::Craft { item, count } => {
            azalea.chat(&format!("/craft {} {}", item, count));
            Ok(Some(format!("Crafting {} {}", count, item)))
        }
        ToolAction::PlaceBlock { block, x, y, z } => {
            let placement = crate::blueprint::types::BlockPlacement::new(
                *x, *y, *z, block.clone(),
            );
            let mut placer = crate::builder::placement::PlayerPlacer::new(azalea.clone());
            match placer.place_block(&placement).await {
                Ok(()) => Ok(Some(format!("Placed {} at ({}, {}, {})", block, x, y, z))),
                Err(e) => Ok(Some(format!("Failed to place {}: {}", block, e))),
            }
        }
        ToolAction::BuildStructure { structure, x, y, z } => {
            let origin = match (x, y, z) {
                (Some(ox), Some(oy), Some(oz)) => (ox, oy, oz),
                _ => {
                    let pos = azalea.position().map_err(|e| format!("No position: {}", e))?;
                    (pos.x as i32, pos.y as i32, pos.z as i32)
                }
            };

            let blueprint_path = std::path::Path::new(structure);
            match crate::blueprint::BlueprintLoader::load_from_file(blueprint_path) {
                Ok(bp) => {
                    let build_executor = crate::builder::BuildExecutor::new(
                        azalea.clone(),
                        bp,
                        origin,
                    );

                    match build_executor.execute().await {
                        Ok(placed) => Ok(Some(format!(
                            "Built {}: {} blocks placed",
                            structure, placed
                        ))),
                        Err(e) => Ok(Some(format!("Build failed: {}", e))),
                    }
                }
                Err(e) => Ok(Some(format!("Failed to load blueprint '{}': {}", structure, e))),
            }
        }
        ToolAction::ScanArea { radius } => {
            azalea.chat(&format!("Scanning area with radius {}", radius));
            Ok(Some(format!("Scanned area radius {}", radius)))
        }
        ToolAction::SortChests => {
            azalea.chat("I can't sort chests automatically yet");
            Ok(Some("Sort chests - not implemented".to_string()))
        }
    }
}

fn get_bot_client() -> Option<BotClient> {
    BOT_CLIENT.read().as_ref().cloned()
}

fn get_azalea_client() -> Option<Client> {
    BOT_CLIENT
        .read()
        .as_ref()
        .and_then(|c| c.azalea_client.read().clone())
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
    BuildStructure { structure: String, x: Option<i32>, y: Option<i32>, z: Option<i32> },
    Reply { message: String },
    ExecuteCommand { command: String },
    ScanArea { radius: u32 },
    GiveItem { player: String, item: String, count: u32 },
    Teleport { x: i32, y: i32, z: i32 },
    SortChests,
    ProtectPlayer { player: String },
}
