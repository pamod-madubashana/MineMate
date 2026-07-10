use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::{AbstractMonster, Health};
use azalea::ecs::query::With;
use azalea::pathfinder::goals::RadiusGoal;
use azalea::pathfinder::PathfinderClientExt;
use azalea::Client;

/// Melee attack range — only attack enemies this close to bot.
const ATTACK_RANGE: f64 = 4.0;

/// Follow distance — how close to stay to the master.
const FOLLOW_RADIUS: f32 = 2.0;

/// Start the guard loop. Runs a background task that:
///
/// 1. Continuously follows the master player.
/// 2. Only attacks enemies when the master or bot takes damage.
/// 3. Keeps totem of undying in off-hand.
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

            // --- Detect damage to bot ---
            let (bot_health_dropped, bot_prev_health) = {
                let health = get_bot_health(&bot).await;
                let prev = last_bot_health;
                let dropped = matches!((prev, health),
                    (Some(p), Some(c)) if c < p);
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

            // --- Follow master ---
            let master_name = master.read().clone();
            if let Some(name) = master_name {
                if let Some(pos) = get_player_position(&bot, &name).await {
                    bot.start_goto(RadiusGoal {
                        pos,
                        radius: FOLLOW_RADIUS,
                    });
                }
            }

            // --- Reactive combat: only attack if someone took damage ---
            if master_health_dropped || bot_health_dropped {
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
                                if dist_sq <= ATTACK_RANGE * ATTACK_RANGE {
                                    Some((e.clone(), dist_sq))
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                            .map(|(e, _)| e)
                    });

                if let Some(target) = nearest_hostile {
                    tracing::info!("Retaliating against nearby hostile");
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
/// Called only when health drops (totem consumed).
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
