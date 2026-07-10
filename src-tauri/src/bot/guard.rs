use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::AbstractMonster;
use azalea::ecs::query::With;
use azalea::Client;

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

            // Attack nearest hostile entity using native azalea API
            if let Ok(Some(target)) = bot.nearest_entity_by::<(), With<AbstractMonster>>(|_| true) {
                let _ = target.look_at();
                target.attack();
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}
