use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::entity::metadata::{AbstractMonster, Health};
use azalea::ecs::query::With;
use azalea::Client;

/// Detection radius: scan for hostile entities within this range.
const GUARD_RADIUS: f64 = 30.0;

/// Melee attack range.
const ATTACK_RANGE: f64 = 4.0;

/// Health threshold below which the bot equips a totem (if available).
const TOTEM_HEALTH_THRESHOLD: f32 = 6.0;

/// Start the guard loop. Runs a background task that:
///
/// 1. Attacks any hostile entity within `GUARD_RADIUS` when the master
///    or the bot itself takes damage (retaliation).
/// 2. Attacks proactively when a hostile is within `ATTACK_RANGE`.
/// 3. Auto-equips a Totem of Undying when the bot's health drops low,
///    and requests a new one via `/give` if it's consumed.
pub fn start_guard_loop(
    bot: Client,
    guarding_flag: Arc<AtomicBool>,
    master: Arc<parking_lot::RwLock<Option<String>>>,
) {
    tokio::task::spawn(async move {
        let mut last_master_health: Option<f32> = None;
        let mut last_bot_health: Option<f32> = None;

        loop {
            if !guarding_flag.load(Ordering::Relaxed) {
                last_master_health = None;
                last_bot_health = None;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }

            // --- Detect damage to master or bot ---
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

            let bot_health_dropped = {
                let health = get_bot_health(&bot).await;
                let dropped = matches!((last_bot_health, health),
                    (Some(prev), Some(curr)) if curr < prev);
                last_bot_health = health;
                dropped
            };

            let any_health_dropped = master_health_dropped || bot_health_dropped;

            // --- Auto-equip totem when low health ---
            if let Some(hp) = last_bot_health {
                if hp <= TOTEM_HEALTH_THRESHOLD {
                    ensure_totem_equipped(&bot).await;
                }
            }

            // --- Find nearest hostile and decide whether to attack ---
            if let Ok(Some(target)) = bot.nearest_entity_by::<(), With<AbstractMonster>>(|_| true)
            {
                let distance_sq = match (bot.position(), target.position()) {
                    (Ok(bot_pos), Ok(target_pos)) => {
                        let dx = bot_pos.x - target_pos.x;
                        let dy = bot_pos.y - target_pos.y;
                        let dz = bot_pos.z - target_pos.z;
                        dx * dx + dy * dy + dz * dz
                    }
                    _ => f64::MAX,
                };

                let in_melee_range = distance_sq <= ATTACK_RANGE * ATTACK_RANGE;
                let in_guard_radius = distance_sq <= GUARD_RADIUS * GUARD_RADIUS;

                if in_melee_range || (in_guard_radius && any_health_dropped) {
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

/// Find a totem of undying in the bot's inventory and equip it to the
/// off-hand. If none is found, request one via `/give`.
async fn ensure_totem_equipped(bot: &Client) {
    let menu = match bot.menu() {
        Ok(m) => m,
        Err(_) => return,
    };

    // Search inventory for a totem of undying
    let totem_slot = menu.slots().iter().enumerate().find_map(|(i, slot)| {
        let item = slot.lock();
        if item.is_empty() {
            return None;
        }
        let kind = item.kind();
        // "totem_of_undying" — check the registry name
        let name = format!("{:?}", kind);
        if name.contains("TotemOfUndying") {
            Some(i)
        } else {
            None
        }
    });

    match totem_slot {
        Some(slot_index) => {
            // Swap the totem into the off-hand slot (slot 40 in Player inventory).
            // Use a QuickMove (shift-click) or Swap operation to move it to offhand.
            // For simplicity, use the /replaceitem command via chat.
            bot.chat(&format!(
                "/replaceitem entity @s container.{} totem_of_undying",
                slot_index
            ));
        }
        None => {
            // No totem found — request one
            bot.chat("/give @s totem_of_undying");
        }
    }
}
