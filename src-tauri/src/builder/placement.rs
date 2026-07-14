use azalea::Client;
use azalea::pathfinder::PathfinderClientExt;
use azalea::pathfinder::goals::RadiusGoal;
use azalea::registry::builtin::ItemKind;
use crate::blueprint::types::BlockPlacement;
use crate::bot::pathfinding;
use super::creative_manager::CreativeInventoryManager;

const PLACE_DELAY_MS: u64 = 40;
const REACH_DISTANCE: f64 = 4.5;
const WALK_TIMEOUT_SECS: u64 = 120;
const PATHFIND_RADIUS: f32 = 2.0;
const FLY_SPEED: f64 = 1.5;

fn normalize_block_id(id: &str) -> String {
    id.trim().to_lowercase()
}

fn block_id_to_item_kind(block_id: &str) -> Result<ItemKind, String> {
    let normalized = normalize_block_id(block_id);
    let with_prefix = if normalized.starts_with("minecraft:") {
        normalized.clone()
    } else {
        format!("minecraft:{}", normalized)
    };

    with_prefix.parse().map_err(|_| format!("Unknown block/item: {}", block_id))
}

pub struct PlayerPlacer {
    bot: Client,
    creative_manager: CreativeInventoryManager,
    current_item: Option<String>,
}

impl PlayerPlacer {
    pub fn new(bot: Client) -> Self {
        let creative_manager = CreativeInventoryManager::new(bot.clone());
        Self {
            bot,
            creative_manager,
            current_item: None,
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

    async fn ensure_item(&mut self, block_id: &str) -> Result<(), String> {
        let normalized = normalize_block_id(block_id);

        if self.current_item.as_deref() == Some(&normalized) {
            return Ok(());
        }

        let item_kind = block_id_to_item_kind(block_id)?;
        self.creative_manager.inject_item(item_kind, 1).await?;
        self.creative_manager.select_hotbar_slot();
        self.current_item = Some(normalized);

        tracing::debug!("Injected {} into hotbar slot 0", block_id);
        Ok(())
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

    fn is_air_block(&self, pos: &azalea::BlockPos) -> bool {
        let world = match self.bot.world() {
            Ok(w) => w,
            Err(_) => return false,
        };
        let w = world.read();
        match w.get_block_state(*pos) {
            Some(state) => state.is_air(),
            None => true,
        }
    }

    async fn break_block_at(&self, pos: &azalea::BlockPos) -> Result<(), String> {
        let adj = self.find_adjacent_position(pos)
            .ok_or("No adjacent position to break block")?;
        self.walk_to_block(&adj).await?;

        self.bot.look_at(pos.center());
        self.bot.wait_updates(1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        self.bot.mine(*pos).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    async fn fly_to_position(&self, target: &azalea::BlockPos) -> Result<(), String> {
        let target_vec = azalea::Vec3::new(
            target.x as f64 + 0.5,
            target.y as f64,
            target.z as f64 + 0.5,
        );

        let start = tokio::time::Instant::now();
        let mut last_progress = tokio::time::Instant::now();
        let mut last_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;

        self.bot.start_goto_with_opts(
            RadiusGoal { pos: target_vec, radius: PATHFIND_RADIUS },
            pathfinding::smart_pathfinder_opts(),
        );

        loop {
            if start.elapsed() > tokio::time::Duration::from_secs(WALK_TIMEOUT_SECS) {
                tracing::warn!("Fly timeout after {}s, skipping", WALK_TIMEOUT_SECS);
                return Err("Fly timeout".into());
            }

            let bot_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;
            let dx = bot_pos.x - (target.x as f64 + 0.5);
            let dy = bot_pos.y - target.y as f64;
            let dz = bot_pos.z - target.z as f64;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist <= REACH_DISTANCE {
                break;
            }

            let moved = (
                (bot_pos.x - last_pos.x).powi(2) +
                (bot_pos.y - last_pos.y).powi(2) +
                (bot_pos.z - last_pos.z).powi(2)
            ).sqrt();

            if moved > 0.1 {
                last_progress = tokio::time::Instant::now();
                last_pos = bot_pos;
            } else if last_progress.elapsed() > tokio::time::Duration::from_secs(10) {
                tracing::warn!("No progress for 10s, retrying pathfind");
                self.bot.stop_pathfinding();
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                self.bot.start_goto_with_opts(
                    RadiusGoal { pos: target_vec, radius: PATHFIND_RADIUS },
                    pathfinding::smart_pathfinder_opts(),
                );
                last_progress = tokio::time::Instant::now();
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn walk_to_block(&self, target: &azalea::BlockPos) -> Result<(), String> {
        let pos = azalea::Vec3::new(
            target.x as f64 + 0.5,
            target.y as f64,
            target.z as f64 + 0.5,
        );

        self.bot.start_goto_with_opts(
            RadiusGoal { pos, radius: PATHFIND_RADIUS },
            pathfinding::smart_pathfinder_opts(),
        );

        let start = tokio::time::Instant::now();
        let mut last_progress = tokio::time::Instant::now();
        let mut last_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;

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

            let moved = (
                (bot_pos.x - last_pos.x).powi(2) +
                (bot_pos.y - last_pos.y).powi(2) +
                (bot_pos.z - last_pos.z).powi(2)
            ).sqrt();

            if moved > 0.1 {
                last_progress = tokio::time::Instant::now();
                last_pos = bot_pos;
            } else if last_progress.elapsed() > tokio::time::Duration::from_secs(10) {
                tracing::warn!("No progress for 10s, retrying pathfind");
                self.bot.stop_pathfinding();
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                self.bot.start_goto_with_opts(
                    RadiusGoal { pos, radius: PATHFIND_RADIUS },
                    pathfinding::smart_pathfinder_opts(),
                );
                last_progress = tokio::time::Instant::now();
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn place_block(&mut self, placement: &BlockPlacement) -> Result<(), String> {
        let target = azalea::BlockPos::new(placement.x, placement.y, placement.z);

        self.ensure_item(&placement.block_id).await?;

        let dist = self.distance_to(&target);
        if dist > REACH_DISTANCE {
            let adj = self.find_adjacent_position(&target)
                .ok_or("No adjacent position")?;
            self.walk_to_block(&adj).await?;
        }

        if !self.is_air_block(&target) {
            tracing::debug!("Breaking existing block at ({}, {}, {})", target.x, target.y, target.z);
            self.break_block_at(&target).await?;
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
        let bot_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;
        let bot_block = azalea::BlockPos::new(
            bot_pos.x as i32, bot_pos.y as i32, bot_pos.z as i32,
        );

        let mut sorted: Vec<&BlockPlacement> = placements.iter().collect();
        sorted.sort_by(|a, b| {
            let a_dist = ((a.x - bot_block.x).pow(2) + (a.y - bot_block.y).pow(2) + (a.z - bot_block.z).pow(2)) as f64;
            let b_dist = ((b.x - bot_block.x).pow(2) + (b.y - bot_block.y).pow(2) + (b.z - bot_block.z).pow(2)) as f64;
            a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut placed = 0;
        let mut skipped = 0;

        for (i, placement) in sorted.iter().enumerate() {
            if i % 50 == 0 || i == sorted.len() - 1 {
                tracing::info!(
                    "[{}/{}] Placing {} blocks...",
                    i + 1, sorted.len(), sorted.len() - i
                );
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
