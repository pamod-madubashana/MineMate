use azalea::Client;
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
