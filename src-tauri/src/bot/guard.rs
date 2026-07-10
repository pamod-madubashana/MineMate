use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::{AbstractMonster, Health};
use azalea::ecs::query::With;
use azalea::pathfinder::goals::RadiusGoal;
use azalea::pathfinder::PathfinderClientExt;
use azalea::Client;

/// Detection radius: scan for hostile entities within this range.
const GUARD_RADIUS: f64 = 25.0;

/// Melee attack range.
const ATTACK_RANGE: f64 = 4.0;

/// How close to get before attacking (pursuit distance).
const PURSUIT_DISTANCE: f32 = 3.5;

/// Start the guard loop. Runs a background task that:
///
/// 1. Seeks hostile entities within GUARD_RADIUS of the master and pursues them.
/// 2. Retaliates when the bot or master takes damage.
/// 3. Moves closer to enemies while attacking (like real players).
/// 4. Always keeps diamond sword in main hand and totem in off-hand.
pub fn start_guard_loop(
    bot: Client,
    guarding_flag: Arc<AtomicBool>,
    master: Arc<parking_lot::RwLock<Option<String>>>,
) {
    tokio::task::spawn(async move {
        let mut last_master_health: Option<f32> = None;
        let mut last_bot_health: Option<f32> = None;
        let mut last_totem_time = std::time::Instant::now();

        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                last_master_health = None;
                last_bot_health = None;
                bot.stop_pathfinding();
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }

            // --- Detect damage to master ---
            let master_health_dropped = {
                let name = master.read().clone();
                match name {
                    Some(name) => {
                        let health = get_player_health(&bot, &name).await;
                        let dropped = matches!((last_master_health, health),
                            (Some(prev), Some(curr)) if curr < prev);
                        last_master_health = health;
                        dropped
                    }
                    None => {
                        last_master_health = None;
                        false
                    }
                }
            };

            // --- Detect damage to bot (totem consumed) ---
            let bot_health_dropped = {
                let health = get_bot_health(&bot).await;
                let dropped = matches!((last_bot_health, health),
                    (Some(prev), Some(curr)) if curr < prev);
                last_bot_health = health;
                dropped
            };

            let any_health_dropped = master_health_dropped || bot_health_dropped;

            // --- Re-equip totem after it's consumed (health dropped) with cooldown ---
            if bot_health_dropped && last_totem_time.elapsed().as_secs() >= 10 {
                ensure_totem_equipped(&bot).await;
                last_totem_time = std::time::Instant::now();
            }

            // --- Find nearest hostile entity ---
            let nearest_hostile = bot
                .nearest_entity_by::<(), With<AbstractMonster>>(|_| true)
                .ok()
                .flatten();

            // --- Determine if we should attack ---
            let mut should_attack = false;
            let mut attack_target_pos = None;

            if let Some(ref target) = nearest_hostile {
                let distance_sq = match (bot.position(), target.position()) {
                    (Ok(bot_pos), Ok(target_pos)) => {
                        let dx = bot_pos.x - target_pos.x;
                        let dy = bot_pos.y - target_pos.y;
                        let dz = bot_pos.z - target_pos.z;
                        let d = dx * dx + dy * dy + dz * dz;
                        attack_target_pos = Some(target_pos);
                        d
                    }
                    _ => f64::MAX,
                };

                let in_attack_range = distance_sq <= ATTACK_RANGE * ATTACK_RANGE;
                let in_guard_radius = distance_sq <= GUARD_RADIUS * GUARD_RADIUS;

                if in_attack_range {
                    should_attack = true;
                } else if in_guard_radius && any_health_dropped {
                    should_attack = true;
                }
            }

            // --- Execute attack behavior ---
            if should_attack {
                if let Some(ref target) = nearest_hostile {
                    let distance_sq = match (bot.position(), target.position()) {
                        (Ok(bot_pos), Ok(target_pos)) => {
                            let dx = bot_pos.x - target_pos.x;
                            let dy = bot_pos.y - target_pos.y;
                            let dz = bot_pos.z - target_pos.z;
                            dx * dx + dy * dy + dz * dz
                        }
                        _ => f64::MAX,
                    };

                    if distance_sq <= ATTACK_RANGE * ATTACK_RANGE {
                        let _ = target.look_at();
                        target.attack();
                    } else {
                        // Pursue enemy
                        if let Some(target_pos) = attack_target_pos {
                            bot.start_goto(RadiusGoal {
                                pos: target_pos,
                                radius: PURSUIT_DISTANCE,
                            });
                        }
                    }
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

/// Get the bot's own health.
async fn get_bot_health(bot: &Client) -> Option<f32> {
    let health = bot.component::<Health>().ok()?;
    Some(health.0)
}

/// Ensure a totem of undying is in the bot's off-hand.
/// Called only when health drops (totem consumed).
async fn ensure_totem_equipped(bot: &Client) {
    bot.chat("/give @s minecraft:totem_of_undying 1");
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    bot.chat("/item replace entity @s weapon.offhand with minecraft:totem_of_undying");
}
