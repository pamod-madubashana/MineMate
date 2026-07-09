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

            bot.start_goto(RadiusGoal {
                pos: target_pos,
                radius: 2.0,
            });

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });
}

async fn get_player_position(bot: &Client, name: &str) -> Option<azalea::Vec3> {
    let uuid = bot.player_uuid_by_username(name).ok()??;
    let entity = bot.entity_by_uuid(uuid)?;
    entity.position().ok()
}
