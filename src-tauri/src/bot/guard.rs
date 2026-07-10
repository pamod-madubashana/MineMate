use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::Client;

/// Start the guard loop. Runs a background task that periodically attacks
/// nearby entities while guarding is enabled.
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

            // Azalea doesn't have a direct entity iterator, so we proxy
            // guard behavior through server commands when possible.
            // The `/attack` command targets the nearest hostile entity.
            bot.chat("/attack");

            // Stay close to master if set
            if let Some(master_name) = master.read().as_ref().cloned() {
                let uuid = bot.player_uuid_by_username(&master_name).ok().flatten();
                if let Some(uuid) = uuid {
                    if let Some(entity) = bot.entity_by_uuid(uuid) {
                        if let Ok(pos) = entity.position() {
                            let dist = pos.distance_to(bot.entity.borrow().position());
                            if dist > 5.0 {
                                bot.start_goto(azalea::pathfinder::goals::RadiusGoal {
                                    pos,
                                    radius: 3.0,
                                });
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}
