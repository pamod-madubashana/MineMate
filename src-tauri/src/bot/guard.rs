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
        let mut last_master_totem_time = std::time::Instant::now();
        let mut pursuing = false;

        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                last_master_health = None;
                last_bot_health = None;
                pursuing = false;
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
                        let prev = last_master_health;
                        let dropped = matches!((prev, health),
                            (Some(p), Some(c)) if c < p);

                        // Detect master totem consumed (health was low, now increased)
                        if let (Some(prev_hp), Some(curr_hp)) = (prev, health) {
                            if curr_hp > prev_hp
                                && prev_hp <= 10.0
                                && last_master_totem_time.elapsed().as_secs() >= 15
                            {
                                tracing::info!("Master totem likely consumed, giving new one");
                                give_totem_to_player(&bot, &name).await;
                                last_master_totem_time = std::time::Instant::now();
                            }
                        }

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
            let (bot_health_dropped, bot_prev_health) = {
                let health = get_bot_health(&bot).await;
                let prev = last_bot_health;
                let dropped = matches!((prev, health),
                    (Some(p), Some(c)) if c < p);
                last_bot_health = health;
                (dropped, prev)
            };

            let any_health_dropped = master_health_dropped || bot_health_dropped;

            // --- Detect totem consumed on bot (health was low, now increased) ---
            if let (Some(prev_hp), Some(curr_hp)) = (bot_prev_health, last_bot_health) {
                // Health increase while previously low = totem healed us
                if curr_hp > prev_hp && prev_hp <= 10.0 && last_totem_time.elapsed().as_secs() >= 5 {
                    tracing::info!("Bot totem consumed, re-equipping");
                    ensure_totem_equipped(&bot).await;
                    last_totem_time = std::time::Instant::now();
                }
            }

            // --- Find hostile entities near the master ---
            let master_pos = {
                let name = master.read().clone();
                name.and_then(|n| {
                    let uuid = bot.player_uuid_by_username(&n).ok()??;
                    let entity = bot.entity_by_uuid(uuid)?;
                    entity.position().ok()
                })
            };

            // Get all hostile entities and find the closest one to the master
            let nearest_hostile = if let Some(master_position) = master_pos {
                bot.nearest_entities::<With<AbstractMonster>>()
                    .ok()
                    .map(|entities| {
                        entities
                            .iter()
                            .filter_map(|e| {
                                let pos = e.position().ok()?;
                                let dx = master_position.x - pos.x;
                                let dy = master_position.y - pos.y;
                                let dz = master_position.z - pos.z;
                                let dist_sq = dx * dx + dy * dy + dz * dz;
                                if dist_sq <= GUARD_RADIUS * GUARD_RADIUS {
                                    Some((e.clone(), dist_sq))
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                            .map(|(e, _)| e)
                    })
                    .flatten()
            } else {
                None
            };

            // --- Determine behavior ---
            if let Some(ref target) = nearest_hostile {
                // Distance from hostile to master
                let distance_to_master_sq = match (master_pos, target.position()) {
                    (Some(mp), Ok(tp)) => {
                        let dx = mp.x - tp.x;
                        let dy = mp.y - tp.y;
                        let dz = mp.z - tp.z;
                        dx * dx + dy * dy + dz * dz
                    }
                    _ => f64::MAX,
                };

                // Distance from bot to hostile
                let distance_to_bot_sq = match (bot.position(), target.position()) {
                    (Ok(bp), Ok(tp)) => {
                        let dx = bp.x - tp.x;
                        let dy = bp.y - tp.y;
                        let dz = bp.z - tp.z;
                        dx * dx + dy * dy + dz * dz
                    }
                    _ => f64::MAX,
                };

                let in_attack_range = distance_to_bot_sq <= ATTACK_RANGE * ATTACK_RANGE;
                let near_master = distance_to_master_sq <= GUARD_RADIUS * GUARD_RADIUS;

                // Pursue if enemy is near master or we're already attacking
                if near_master || pursuing {
                    pursuing = true;

                    if in_attack_range {
                        let _ = target.look_at();
                        target.attack();
                    } else {
                        // Pursue enemy
                        if let Ok(target_pos) = target.position() {
                            bot.start_goto(RadiusGoal {
                                pos: target_pos,
                                radius: PURSUIT_DISTANCE,
                            });
                        }
                    }
                }
            } else {
                // No hostile near master — stop pursuing
                if pursuing {
                    bot.stop_pathfinding();
                    pursuing = false;
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

/// Equip a totem of undying directly into a player's off-hand.
async fn give_totem_to_player(bot: &Client, player: &str) {
    bot.chat(&format!(
        "/item replace entity {} weapon.offhand with minecraft:totem_of_undying",
        player
    ));
}
