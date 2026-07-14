use azalea::Client;
use azalea::pathfinder::PathfinderClientExt;
use crate::blueprint::types::Blueprint;
use super::planner::plan_build;
use super::placement::PlayerPlacer;

pub struct BuildExecutor {
    bot: Client,
    blueprint: Blueprint,
    origin: (i32, i32, i32),
}

impl BuildExecutor {
    pub fn new(bot: Client, blueprint: Blueprint, origin: (i32, i32, i32)) -> Self {
        Self { bot, blueprint, origin }
    }

    pub async fn execute(&self) -> Result<u32, String> {
        let plan = plan_build(&self.blueprint, self.origin);
        tracing::info!("Starting build: {} blocks, {} layers", plan.total_blocks, plan.layers);

        self.bot.stop_pathfinding();
        if let Some(bc) = crate::bot::handler::BOT_CLIENT.read().as_ref() {
            bc.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
            bc.set_guarding(false);
            bc.set_following(None);
            bc.set_master(None);
        }

        let bot_pos = self.bot.position().map_err(|e| format!("No position: {}", e))?;

        if let Some(first) = plan.placements.first() {
            let first_target = azalea::BlockPos::new(first.x, first.y, first.z);
            let dist = (
                (bot_pos.x - first_target.x as f64).powi(2) +
                (bot_pos.y - first_target.y as f64).powi(2) +
                (bot_pos.z - first_target.z as f64).powi(2)
            ).sqrt();

            if dist > 20.0 {
                tracing::info!("Teleporting to build area ({}, {}, {})", first.x, first.y, first.z);
                self.bot.chat(&format!("/tp {} {} {}", first.x, first.y, first.z));
                self.bot.wait_updates(20).await;
                tracing::info!("Waiting for chunks to load...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        let mut placer = PlayerPlacer::new(self.bot.clone());
        placer.ensure_creative().await?;

        let mut total_placed = 0;
        let mut current_y = i32::MIN;

        for chunk in plan.placements.chunks(100) {
            if let Some(first) = chunk.first() {
                if first.y != current_y {
                    current_y = first.y;
                    tracing::info!("Building layer y={}", current_y);
                }
            }

            match placer.place_blocks(chunk).await {
                Ok(placed) => { total_placed += placed; }
                Err(e) => {
                    tracing::error!("Build error: {}", e);
                    return Err(e);
                }
            }
        }

        self.bot.chat("/gamemode survival @s");
        self.bot.wait_updates(3).await;
        tracing::info!("Build complete: {}/{} blocks placed, back to survival", total_placed, plan.total_blocks);
        Ok(total_placed)
    }
}
