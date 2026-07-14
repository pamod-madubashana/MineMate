use azalea::Client;
use azalea::pathfinder::PathfinderClientExt;
use azalea::pathfinder::goals::RadiusGoal;
use crate::blueprint::types::BlockPlacement;
use crate::bot::pathfinding;

const PLACE_DELAY_MS: u64 = 40;
const REACH_DISTANCE: f64 = 4.5;
const WALK_TIMEOUT_SECS: u64 = 30;

fn normalize_block_id(id: &str) -> String {
    id.trim().to_lowercase()
}

pub struct PlayerPlacer {
    bot: Client,
    slot_map: std::collections::HashMap<String, u8>,
    next_slot: u8,
}

impl PlayerPlacer {
    pub fn new(bot: Client) -> Self {
        Self {
            bot,
            slot_map: std::collections::HashMap::new(),
            next_slot: 0,
        }
    }

    pub async fn ensure_creative(&self) -> Result<(), String> {
        tracing::info!("Switching to creative mode...");
        self.bot.chat("/gamemode creative @s");
        self.bot.wait_updates(5).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        tracing::info!("Creative mode switch sent");
        Ok(())
    }

    async fn give_block(&mut self, block_id: &str) -> Result<u8, String> {
        let normalized = normalize_block_id(block_id);

        if let Some(&slot) = self.slot_map.get(&normalized) {
            return Ok(slot);
        }

        if self.next_slot >= 9 {
            return Err("Hotbar full (9/9)".into());
        }

        let slot = self.next_slot;
        self.bot.chat(&format!("/give @s {} 1", block_id));
        self.bot.wait_updates(3).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        self.slot_map.insert(normalized, slot);
        self.next_slot += 1;

        tracing::info!("Assigned {} -> hotbar slot {}", block_id, slot);
        Ok(slot)
    }

    fn select_slot(&self, slot: u8) {
        let current = self.bot.selected_hotbar_slot().unwrap_or(0);
        if current != slot {
            self.bot.set_selected_hotbar_slot(slot);
        }
    }

    fn bot_block_pos(&self) -> Option<azalea::BlockPos> {
        let pos = self.bot.position().ok()?;
        Some(azalea::BlockPos::new(pos.x as i32, pos.y as i32, pos.z as i32))
    }

    fn distance_to(&self, target: &azalea::BlockPos) -> f64 {
        let bot_pos = match self.bot.position() {
            Ok(p) => p,
            Err(_) => return f64::MAX,
        };
        let dx = bot_pos.x - (target.x as f64 + 0.5);
        let dy = bot_pos.y - target.y as f64;
        let dz = bot_pos.z - target.z as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn find_adjacent_position(&self, target: &azalea::BlockPos) -> Option<azalea::BlockPos> {
        let bot_block = self.bot_block_pos()?;

        let offsets = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 0, 1),
            (0, 0, -1),
            (0, 1, 0),
            (0, -1, 0),
        ];

        let mut best = None;
        let mut best_dist = f64::MAX;

        for (dx, dy, dz) in offsets {
            let candidate = azalea::BlockPos::new(
                target.x + dx,
                target.y + dy,
                target.z + dz,
            );

            let dist = (
                (candidate.x - bot_block.x).pow(2) +
                (candidate.y - bot_block.y).pow(2) +
                (candidate.z - bot_block.z).pow(2)
            ) as f64;

            if dist < best_dist {
                best_dist = dist;
                best = Some(candidate);
            }
        }

        best
    }

    async fn walk_to_block(&self, target: &azalea::BlockPos) -> Result<(), String> {
        let pos = azalea::Vec3::new(
            target.x as f64 + 0.5,
            target.y as f64,
            target.z as f64 + 0.5,
        );

        self.bot.start_goto_with_opts(
            RadiusGoal { pos, radius: 1.0 },
            pathfinding::smart_pathfinder_opts(),
        );

        let start = tokio::time::Instant::now();
        loop {
            if start.elapsed() > tokio::time::Duration::from_secs(WALK_TIMEOUT_SECS) {
                tracing::warn!("Walk timeout after {}s, skipping", WALK_TIMEOUT_SECS);
                return Err("Walk timeout".into());
            }

            let bot_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;
            let dx = bot_pos.x - (target.x as f64 + 0.5);
            let dy = bot_pos.y - target.y as f64;
            let dz = bot_pos.z - target.z as f64;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist <= REACH_DISTANCE {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn place_block(&mut self, placement: &BlockPlacement) -> Result<(), String> {
        let target = azalea::BlockPos::new(placement.x, placement.y, placement.z);

        let slot = self.give_block(&placement.block_id).await?;
        self.select_slot(slot);

        let dist = self.distance_to(&target);
        if dist > REACH_DISTANCE {
            let adj = self.find_adjacent_position(&target)
                .ok_or("No adjacent position")?;
            self.walk_to_block(&adj).await?;
        }

        self.bot.look_at(target.center());
        self.bot.wait_updates(1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;

        self.bot.block_interact(target);
        self.bot.wait_updates(1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(PLACE_DELAY_MS)).await;

        Ok(())
    }

    pub async fn place_blocks(&mut self, placements: &[BlockPlacement]) -> Result<u32, String> {
        let mut sorted: Vec<&BlockPlacement> = placements.iter().collect();
        sorted.sort_by_key(|p| {
            let target = azalea::BlockPos::new(p.x, p.y, p.z);
            let dist = self.distance_to(&target);
            (dist * 100.0) as i64
        });

        let mut placed = 0;
        let mut last_block_type = String::new();
        let mut skipped = 0;

        for (i, placement) in sorted.iter().enumerate() {
            let normalized = normalize_block_id(&placement.block_id);
            if normalized != last_block_type {
                tracing::info!(
                    "[{}/{}] Placing {} at ({}, {}, {})",
                    i + 1, sorted.len(), placement.block_id,
                    placement.x, placement.y, placement.z
                );
                last_block_type = normalized;
            }

            match self.place_block(placement).await {
                Ok(()) => { placed += 1; }
                Err(e) => {
                    skipped += 1;
                    tracing::warn!(
                        "Skip ({}, {}, {}): {}",
                        placement.x, placement.y, placement.z, e
                    );
                }
            }
        }

        if skipped > 0 {
            tracing::info!("Skipped {} unreachable blocks", skipped);
        }

        Ok(placed)
    }
}

pub async fn place_blocks(bot: &Client, placements: &[BlockPlacement]) -> Result<u32, String> {
    let mut placer = PlayerPlacer::new(bot.clone());
    placer.place_blocks(placements).await
}
