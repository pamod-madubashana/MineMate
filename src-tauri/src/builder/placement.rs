use azalea::Client;
use azalea::pathfinder::PathfinderClientExt;
use azalea::pathfinder::goals::RadiusGoal;
use azalea::registry::builtin::ItemKind;
use crate::blueprint::types::BlockPlacement;
use crate::bot::pathfinding;
use super::creative_manager::CreativeInventoryManager;

const PLACE_DELAY_MS: u64 = 50;
const REACH_DISTANCE: f64 = 4.5;
const FLY_TICK_MS: u64 = 50;
const MOVE_TIMEOUT_SECS: u64 = 30;

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
        Self { bot, creative_manager, current_item: None }
    }

    pub async fn ensure_creative(&mut self) -> Result<(), String> {
        tracing::info!("Switching to creative mode...");
        self.bot.chat("/gamemode creative @s");
        self.bot.wait_updates(10).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        self.current_item = None;
        tracing::info!("Creative mode confirmed");
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
        tracing::debug!("Equipped {}", block_id);
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
        let offsets = [(1,0,0),(-1,0,0),(0,0,1),(0,0,-1),(0,1,0),(0,-1,0)];
        let mut best = None;
        let mut best_dist = f64::MAX;
        for (dx, dy, dz) in offsets {
            let c = azalea::BlockPos::new(target.x+dx, target.y+dy, target.z+dz);
            let d = ((c.x-bot_block.x).pow(2)+(c.y-bot_block.y).pow(2)+(c.z-bot_block.z).pow(2)) as f64;
            if d < best_dist { best_dist = d; best = Some(c); }
        }
        best
    }

    fn is_air_block(&self, pos: &azalea::BlockPos) -> bool {
        let world = match self.bot.world() { Ok(w) => w, Err(_) => return false };
        let w = world.read();
        match w.get_block_state(*pos) {
            Some(state) => state.is_air(),
            None => true,
        }
    }

    async fn fly_towards(&self, target: &azalea::BlockPos) -> Result<(), String> {
        let goal_pos = azalea::Vec3::new(
            target.x as f64 + 0.5,
            target.y as f64,
            target.z as f64 + 0.5,
        );

        self.bot.start_goto_with_opts(
            RadiusGoal { pos: goal_pos, radius: 2.0 },
            pathfinding::smart_pathfinder_opts(),
        );

        let start = tokio::time::Instant::now();
        let mut last_progress = tokio::time::Instant::now();
        let mut last_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;

        loop {
            if start.elapsed() > tokio::time::Duration::from_secs(MOVE_TIMEOUT_SECS) {
                tracing::warn!("Move timeout ({},{},{})", target.x, target.y, target.z);
                self.bot.stop_pathfinding();
                return Err("Move timeout".into());
            }

            let bot_pos = self.bot.position().map_err(|e| format!("No pos: {}", e))?;
            let dx = bot_pos.x - goal_pos.x;
            let dy = bot_pos.y - goal_pos.y;
            let dz = bot_pos.z - goal_pos.z;
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();

            if dist <= REACH_DISTANCE {
                break;
            }

            let moved = ((bot_pos.x-last_pos.x).powi(2)+(bot_pos.y-last_pos.y).powi(2)+(bot_pos.z-last_pos.z).powi(2)).sqrt();
            if moved > 0.1 {
                last_progress = tokio::time::Instant::now();
                last_pos = bot_pos;
            } else if last_progress.elapsed() > tokio::time::Duration::from_secs(8) {
                tracing::warn!("Stuck, restarting pathfind");
                self.bot.stop_pathfinding();
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                self.bot.start_goto_with_opts(
                    RadiusGoal { pos: goal_pos, radius: 2.0 },
                    pathfinding::smart_pathfinder_opts(),
                );
                last_progress = tokio::time::Instant::now();
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(FLY_TICK_MS)).await;
        }

        self.bot.stop_pathfinding();
        Ok(())
    }

    async fn break_block_at(&self, pos: &azalea::BlockPos) -> Result<(), String> {
        let adj = self.find_adjacent_position(pos).ok_or("No adjacent pos")?;
        self.fly_towards(&adj).await?;
        self.bot.look_at(pos.center());
        self.bot.wait_updates(1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        self.bot.mine(*pos).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    pub async fn place_block(&mut self, placement: &BlockPlacement) -> Result<(), String> {
        let target = azalea::BlockPos::new(placement.x, placement.y, placement.z);
        self.ensure_item(&placement.block_id).await?;

        let dist = self.distance_to(&target);
        if dist > REACH_DISTANCE {
            let adj = self.find_adjacent_position(&target).ok_or("No adjacent pos")?;
            self.fly_towards(&adj).await?;
        }

        if !self.is_air_block(&target) {
            tracing::debug!("Breaking at ({},{},{})", target.x, target.y, target.z);
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
        let bx = bot_pos.x as i32;
        let by = bot_pos.y as i32;
        let bz = bot_pos.z as i32;

        let mut sorted: Vec<&BlockPlacement> = placements.iter().collect();
        sorted.sort_by(|a, b| {
            let ad = (a.x-bx).pow(2)+(a.y-by).pow(2)+(a.z-bz).pow(2);
            let bd = (b.x-bx).pow(2)+(b.y-by).pow(2)+(b.z-bz).pow(2);
            ad.cmp(&bd)
        });

        let mut placed = 0u32;
        let mut skipped = 0u32;
        for (i, p) in sorted.iter().enumerate() {
            if i % 100 == 0 {
                tracing::info!("[{}/{}] blocks remaining", i, sorted.len());
            }
            match self.place_block(p).await {
                Ok(()) => placed += 1,
                Err(e) => {
                    skipped += 1;
                    if skipped <= 10 {
                        tracing::warn!("Skip ({},{},{}): {}", p.x, p.y, p.z, e);
                    }
                }
            }
        }
        if skipped > 0 { tracing::info!("Skipped {} blocks", skipped); }
        Ok(placed)
    }
}

pub async fn place_blocks(bot: &Client, placements: &[BlockPlacement]) -> Result<u32, String> {
    let mut placer = PlayerPlacer::new(bot.clone());
    placer.place_blocks(placements).await
}
