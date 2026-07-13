use azalea::pathfinder::PathfinderClientExt;

use crate::bot::handler::BOT_CLIENT;

#[derive(Debug, Clone)]
pub enum Command {
    Follow { player: String },
    Stop,
    Mine { block: String, count: u32 },
    Move { x: i32, y: i32, z: i32 },
    Guard { player: String },
    Come,
    Where,
    Blueprints,
    BuildBlueprint { index: usize },
    ImportBlueprint { url: String, name: Option<String> },
    Help,
}

pub fn parse_command(message: &str, prefix: &str) -> Option<Command> {
    let message = message.trim();
    if !message.starts_with(prefix) {
        return None;
    }

    let args: Vec<&str> = message[prefix.len()..].split_whitespace().collect();
    if args.is_empty() {
        return None;
    }

    match args[0].to_lowercase().as_str() {
        "follow" => {
            let player = args.get(1)?.to_string();
            Some(Command::Follow { player })
        }
        "stop" => Some(Command::Stop),
        "mine" => {
            let block = args.get(1)?.to_string();
            let count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
            Some(Command::Mine { block, count })
        }
        "move" | "goto" => {
            let x = args.get(1)?.parse().ok()?;
            let y = args.get(2)?.parse().ok()?;
            let z = args.get(3)?.parse().ok()?;
            Some(Command::Move { x, y, z })
        }
        "guard" | "protect" => {
            let player = args.get(1)?.to_string();
            Some(Command::Guard { player })
        }
        "come" => Some(Command::Come),
        "where" | "pos" => Some(Command::Where),
        "blueprints" | "bp" => Some(Command::Blueprints),
        "build" => {
            if args.get(1).map(|s| *s == "blueprint").unwrap_or(false) {
                let index = args.get(2)?.parse().ok()?;
                Some(Command::BuildBlueprint { index })
            } else {
                None
            }
        }
        "import" => {
            let url = args.get(1)?.to_string();
            let name = args.get(2).map(|s| s.to_string());
            Some(Command::ImportBlueprint { url, name })
        }
        "help" => Some(Command::Help),
        _ => None,
    }
}

pub async fn execute_command(sender: &str, command: Command) -> Option<String> {
    let bot_client = BOT_CLIENT.read().as_ref().cloned()?;
    let azalea = bot_client.azalea_client.read().clone()?;

    match command {
        Command::Follow { player } => {
            bot_client.follow_stop.store(false, std::sync::atomic::Ordering::Relaxed);
            bot_client.set_following(Some(player.clone()));
            crate::bot::follow::start_following(
                azalea.clone(),
                player.clone(),
                bot_client.follow_stop.clone(),
            );
            Some(format!("Now following {}", player))
        }
        Command::Stop => {
            azalea.stop_pathfinding();
            bot_client.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
            bot_client.set_guarding(false);
            bot_client.set_following(None);
            Some("Stopped all actions".to_string())
        }
        Command::Mine { block, count } => {
            Some(format!("Mining {}x {} - use AI mode for this", count, block))
        }
        Command::Move { x, y, z } => {
            crate::bot::pathfinding::open_nearby_doors(&azalea, 3).await;
            let pos = azalea::Vec3::new(x as f64, y as f64, z as f64);
            azalea.start_goto_with_opts(
                azalea::pathfinder::goals::RadiusGoal { pos, radius: 1.0 },
                crate::bot::pathfinding::smart_pathfinder_opts(),
            );
            Some(format!("Moving to ({}, {}, {})", x, y, z))
        }
        Command::Guard { player } => {
            bot_client.set_guarding(true);
            bot_client.set_master(Some(player.clone()));
            azalea.chat(&format!("I will protect you, {}!", player));
            Some(format!("Now protecting {}", player))
        }
        Command::Come => {
            let uuid = azalea.player_uuid_by_username(sender).ok()??;
            let entity = azalea.entity_by_uuid(uuid)?;
            let pos = entity.position().ok()?;
            crate::bot::pathfinding::open_nearby_doors(&azalea, 3).await;
            azalea.start_goto_with_opts(
                azalea::pathfinder::goals::RadiusGoal { pos, radius: 2.0 },
                crate::bot::pathfinding::smart_pathfinder_opts(),
            );
            Some(format!("Coming to you, {}!", sender))
        }
        Command::Where => {
            let pos = azalea.position().ok()?;
            Some(format!("I'm at ({:.0}, {:.0}, {:.0})", pos.x, pos.y, pos.z))
        }
        Command::Blueprints => {
            let cache = crate::blueprint::importers::grabcraft::cache::BlueprintCache::new();
            let list = cache.list();
            if list.is_empty() {
                Some("No blueprints imported yet. Use !import <url> to add one.".to_string())
            } else {
                let formatted: Vec<String> = list.iter().enumerate()
                    .map(|(i, name)| format!("{}.{}", i + 1, name))
                    .collect();
                Some(format!("Blueprints: {}", formatted.join(", ")))
            }
        }
        Command::BuildBlueprint { index } => {
            let cache = crate::blueprint::importers::grabcraft::cache::BlueprintCache::new();
            let list = cache.list();
            if list.is_empty() {
                return Some("No blueprints imported.".to_string());
            }
            if index == 0 || index > list.len() {
                return Some(format!("Invalid index. Use 1-{}", list.len()));
            }
            let name = &list[index - 1];
            match cache.get(name) {
                Some(blueprint) => {
                    let pos = azalea.position().ok()?;
                    let origin = (pos.x as i32, pos.y as i32, pos.z as i32);
                    let mut build_executor = crate::builder::BuildExecutor::new(
                        azalea.clone(),
                        blueprint.clone(),
                        origin,
                    );
                    match build_executor.execute().await {
                        Ok(placed) => Some(format!("Built {}: {} blocks placed", name, placed)),
                        Err(e) => Some(format!("Build failed: {}", e)),
                    }
                }
                None => Some(format!("Blueprint '{}' not found", name)),
            }
        }
        Command::ImportBlueprint { url, name } => {
            let importer = crate::blueprint::importers::GrabCraftImporter::new();
            match importer.import(&url).await {
                Ok(blueprint) => {
                    let cache = crate::blueprint::importers::grabcraft::cache::BlueprintCache::new();
                    let save_name = name.unwrap_or_else(|| blueprint.name.clone());
                    let _ = cache.save(&save_name, &blueprint);
                    Some(format!("Imported '{}' ({}x{}x{})", blueprint.name, blueprint.width, blueprint.height, blueprint.length))
                }
                Err(e) => Some(format!("Import failed: {}", e)),
            }
        }
        Command::Help => {
            Some(
                "Commands: !follow <player>, !stop, !mine <block> <count>, \
                 !move <x> <y> <z>, !guard <player>, !come, !where, \
                 !blueprints, !build blueprint <n>, !import <url>, !help"
                    .to_string(),
            )
        }
    }
}
