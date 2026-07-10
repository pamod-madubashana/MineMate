use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::AbstractMonster;
use azalea::ecs::query::With;
use azalea::Client;

/// Maximum distance (blocks) at which the bot will attack a hostile entity.
const ATTACK_RANGE: f64 = 4.0;

/// Start the guard loop. Runs a background task that periodically attacks
/// nearby hostile entities while guarding is enabled.
///
/// Movement (follow / stay-near-master) is handled by the follow loop —
/// this loop only attacks. This avoids conflicting pathfinder goals
/// between guard and follow loops.
pub fn start_guard_loop(
    bot: Client,
    guarding_flag: Arc<AtomicBool>,
    _master: Arc<parking_lot::RwLock<Option<String>>>,
) {
    tokio::task::spawn(async move {
        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }

            // Find nearest hostile entity, then check distance on the returned EntityRef.
            if let Ok(Some(target)) = bot.nearest_entity_by::<(), With<AbstractMonster>>(|_| true)
            {
                let in_range = match (bot.position(), target.position()) {
                    (Ok(bot_pos), Ok(target_pos)) => {
                        let dx = bot_pos.x - target_pos.x;
                        let dy = bot_pos.y - target_pos.y;
                        let dz = bot_pos.z - target_pos.z;
                        (dx * dx + dy * dy + dz * dz) <= ATTACK_RANGE * ATTACK_RANGE
                    }
                    _ => false,
                };

                if in_range {
                    let _ = target.look_at();
                    target.attack();
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}
