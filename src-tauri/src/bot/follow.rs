use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::pathfinder::goals::RadiusGoal;
use azalea::pathfinder::PathfinderClientExt;
use azalea::Client;

/// Start following a player by name. Runs a background loop that periodically
/// updates the pathfinder target. Returns immediately.
pub fn start_following(bot: Client, player_name: String, should_stop: Arc<AtomicBool>) {
    tokio::task::spawn(async move {
        let name = player_name.clone();
        let mut last_target: Option<azalea::Vec3> = None;
        loop {
            if should_stop.load(Ordering::Relaxed) {
                bot.stop_pathfinding();
                return;
            }

            let target_pos = match get_player_position(&bot, &name).await {
                Some(pos) => pos,
                None => {
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                    continue;
                }
            };

            // Only re-path if the target moved significantly or we reached the destination
            let should_repath = match last_target {
                Some(last) => {
                    let dx = target_pos.x - last.x;
                    let dz = target_pos.z - last.z;
                    dx * dx + dz * dz > 4.0 || bot.is_goto_target_reached()
                }
                None => true,
            };

            if should_repath {
                last_target = Some(target_pos);
                bot.start_goto(RadiusGoal {
                    pos: target_pos,
                    radius: 2.0,
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}

async fn get_player_position(bot: &Client, name: &str) -> Option<azalea::Vec3> {
    let uuid = bot.player_uuid_by_username(name).ok()??;
    let entity = bot.entity_by_uuid(uuid)?;
    entity.position().ok()
}
