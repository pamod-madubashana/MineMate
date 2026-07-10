use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::AbstractMonster;
use azalea::pathfinder::PathfinderClientExt;
use azalea::ecs::query::With;
use azalea::Client;

/// Start the guard loop. Runs a background task that periodically attacks
/// nearby entities while guarding is enabled and stays near the master.
pub fn start_guard_loop(
    bot: Client,
    guarding_flag: Arc<AtomicBool>,
    master: Arc<parking_lot::RwLock<Option<String>>>,
) {
    tokio::task::spawn(async move {
        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }

            // Attack nearest hostile entity using native azalea API
            if let Ok(Some(target)) = bot.nearest_entity_by::<(), With<AbstractMonster>>(|_| true) {
                let _ = target.look_at();
                target.attack();
            }

            // Stay near master if set
            if let Some(master_name) = master.read().as_ref().cloned() {
                let uuid = bot.player_uuid_by_username(&master_name).ok().flatten();
                if let Some(uuid) = uuid {
                    if let Some(entity) = bot.entity_by_uuid(uuid) {
                        if let Ok(pos) = entity.position() {
                            bot.start_goto(azalea::pathfinder::goals::RadiusGoal {
                                pos,
                                radius: 3.0,
                            });
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}
