use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::{AbstractMonster, Health};
use azalea::ecs::query::With;
use azalea::Client;

/// Maximum distance (blocks) at which the bot will attack a hostile entity.
const ATTACK_RANGE: f64 = 4.0;

/// Start the guard loop. Runs a background task that periodically attacks
/// nearby hostile entities while guarding is enabled.
///
/// Also monitors the master player's health — when it drops, the bot
/// attacks the nearest hostile entity within range (retaliation mode).
pub fn start_guard_loop(
    bot: Client,
    guarding_flag: Arc<AtomicBool>,
    master: Arc<parking_lot::RwLock<Option<String>>>,
) {
    tokio::task::spawn(async move {
        let mut last_master_health: Option<f32> = None;

        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                last_master_health = None;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }

            // --- Retaliation: if master took damage, attack nearest hostile ---
            let master_health_dropped = {
                let name = master.read().clone();
                match name {
                    Some(name) => {
                        let health = get_player_health(&bot, &name).await;
                        match (last_master_health, health) {
                            (Some(prev), Some(current)) if current < prev => true,
                            _ => false,
                        }
                    }
                    None => false,
                }
            };

            // Update tracked health for next iteration
            {
                let name = master.read().clone();
                if let Some(name) = name {
                    last_master_health = get_player_health(&bot, &name).await;
                }
            }

            // --- Proactive: attack nearest hostile if in range (existing behavior) ---
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

                if in_range || master_health_dropped {
                    let _ = target.look_at();
                    target.attack();
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });
}

/// Get a player's health from their entity metadata.
async fn get_player_health(bot: &Client, name: &str) -> Option<f32> {
    let uuid = bot.player_uuid_by_username(name).ok()??;
    let entity = bot.entity_by_uuid(uuid)?;
    let health = entity.component::<Health>().ok()?;
    Some(health.0)
}
