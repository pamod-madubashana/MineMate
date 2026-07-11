use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::{AbstractMonster, Health};
use azalea::ecs::query::With;
use azalea::pathfinder::goals::RadiusGoal;
use azalea::pathfinder::PathfinderClientExt;
use azalea::Client;

/// Follow distance — how close to stay to the master.
const FOLLOW_RADIUS: f32 = 10.0;

/// How long to chase an enemy after damage is detected (seconds).
/// Long enough to chase down and kill the enemy.
const COMBAT_DURATION: u64 = 30;

/// How far the master must move before we consider them "moving".
const MASTER_MOVE_THRESHOLD: f64 = 1.5;

/// Random idle movement range around the master when standing still.
const IDLE_WANDER_RANGE: f32 = 3.0;

/// Start the guard loop. Runs a background task that:
///
/// 1. Continuously follows the master player.
/// 2. Attacks ANY enemy when the master or bot takes damage (no distance limit).
/// 3. Chases the enemy until dead or COMBAT_DURATION expires.
/// 4. Returns to following master after combat ends.
/// 5. Keeps totem of undying in off-hand.
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
        let mut combat_until: Option<std::time::Instant> = None;
        let mut last_master_position: Option<azalea::Vec3> = None;

        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                last_master_health = None;
                last_bot_health = None;
                combat_until = None;
                bot.stop_pathfinding();
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }

            let now = std::time::Instant::now();

            // --- Detect damage to master ---
            let master_health_dropped = {
                let name = master.read().clone();
                match name {
                    Some(name) => {
                        let health = get_player_health(&bot, &name).await;
                        let prev = last_master_health;
                        let dropped = matches!((prev, health),
                            (Some(p), Some(c)) if c < p);
                        if dropped {
                            tracing::info!("Master took damage: {:.1} -> {:.1}", prev.unwrap_or(0.0), health.unwrap_or(0.0));
                        }

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

            // --- Detect damage to bot ---
            let (bot_health_dropped, bot_prev_health) = {
                let health = get_bot_health(&bot).await;
                let prev = last_bot_health;
                let dropped = matches!((prev, health),
                    (Some(p), Some(c)) if c < p);
                if dropped {
                    tracing::info!("Bot took damage: {:.1} -> {:.1}", prev.unwrap_or(0.0), health.unwrap_or(0.0));
                }
                last_bot_health = health;
                (dropped, prev)
            };

            // --- Detect totem consumed on bot (health was low, now increased) ---
            if let (Some(prev_hp), Some(curr_hp)) = (bot_prev_health, last_bot_health) {
                if curr_hp > prev_hp && prev_hp <= 10.0 && last_totem_time.elapsed().as_secs() >= 5 {
                    tracing::info!("Bot totem consumed, re-equipping");
                    ensure_totem_equipped(&bot).await;
                    last_totem_time = std::time::Instant::now();
                }
            }

            // --- Enter combat mode when damage detected ---
            if master_health_dropped || bot_health_dropped {
                combat_until = Some(now + std::time::Duration::from_secs(COMBAT_DURATION));
                tracing::info!("Combat triggered - chasing enemy");
            }

            // --- Combat: chase and attack enemy (no distance limit) ---
            let in_combat = combat_until.map_or(false, |t| now < t);

            if in_combat {
                // Find nearest hostile ANYWHERE (no distance filter)
                let nearest_hostile = bot
                    .nearest_entities::<With<AbstractMonster>>()
                    .ok()
                    .and_then(|entities| {
                        entities
                            .iter()
                            .filter_map(|e| {
                                let pos = e.position().ok()?;
                                let bot_pos = bot.position().ok()?;
                                let dx = bot_pos.x - pos.x;
                                let dy = bot_pos.y - pos.y;
                                let dz = bot_pos.z - pos.z;
                                let dist_sq = dx * dx + dy * dy + dz * dz;
                                Some((e.clone(), dist_sq))
                            })
                            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                            .map(|(e, _)| e)
                    });

                if let Some(target) = nearest_hostile {
                    if let Ok(target_pos) = target.position() {
                        let bot_pos = bot.position().unwrap_or_default();
                        let dx = bot_pos.x - target_pos.x;
                        let dy = bot_pos.y - target_pos.y;
                        let dz = bot_pos.z - target_pos.z;
                        let dist_sq = dx * dx + dy * dy + dz * dz;

                        if dist_sq <= 4.0 * 4.0 {
                            // In melee range — attack
                            tracing::debug!("Attacking enemy at distance {:.1}", dist_sq.sqrt());
                            let _ = target.look_at();
                            target.attack();
                        } else {
                            // Chase the enemy (any distance)
                            tracing::debug!("Chasing enemy at distance {:.1}", dist_sq.sqrt());
                            bot.start_goto(RadiusGoal {
                                pos: target_pos,
                                radius: 2.0,
                            });
                        }
                    }
                } else {
                    // No enemy found, end combat early
                    tracing::debug!("No hostiles found, ending combat");
                    combat_until = None;
                    bot.stop_pathfinding();
                }
            } else {
                // --- Follow master (when not in combat) ---
                let master_name = master.read().clone();
                if let Some(name) = master_name {
                    if let Some(pos) = get_player_position(&bot, &name).await {
                        // Check if master has moved
                        let master_moved = match last_master_position {
                            Some(last) => {
                                let dx = pos.x - last.x;
                                let dy = pos.y - last.y;
                                let dz = pos.z - last.z;
                                (dx * dx + dy * dy + dz * dz) > MASTER_MOVE_THRESHOLD * MASTER_MOVE_THRESHOLD
                            }
                            None => true,
                        };

                        if master_moved {
                            // Master moved — follow directly
                            last_master_position = Some(pos);
                            bot.start_goto(RadiusGoal {
                                pos,
                                radius: FOLLOW_RADIUS,
                            });
                        } else {
                            // Master standing still — wander randomly nearby
                            let now_millis = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis();
                            let seed = now_millis as f64;
                            let dx = ((seed * 0.123).sin() * 2.0 - 1.0) * IDLE_WANDER_RANGE as f64;
                            let dz = ((seed * 0.456).cos() * 2.0 - 1.0) * IDLE_WANDER_RANGE as f64;
                            let wander_pos = azalea::Vec3::new(
                                pos.x + dx,
                                pos.y,
                                pos.z + dz,
                            );
                            bot.start_goto(RadiusGoal {
                                pos: wander_pos,
                                radius: 1.0,
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

/// Get a player's position.
async fn get_player_position(bot: &Client, name: &str) -> Option<azalea::Vec3> {
    let uuid = bot.player_uuid_by_username(name).ok()??;
    let entity = bot.entity_by_uuid(uuid)?;
    entity.position().ok()
}

/// Ensure a totem of undying is in the bot's off-hand.
async fn ensure_totem_equipped(bot: &Client) {
    bot.chat("/item replace entity @s weapon.offhand with minecraft:totem_of_undying");
}

/// Equip a totem of undying directly into a player's off-hand.
async fn give_totem_to_player(bot: &Client, player: &str) {
    bot.chat(&format!(
        "/item replace entity {} weapon.offhand with minecraft:totem_of_undying",
        player
    ));
}
