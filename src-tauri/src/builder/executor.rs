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

        let bot_pos = self.bot.position().map_err(|e| format!("No position: {}", e))?;
        let dist = (
            (bot_pos.x - self.origin.0 as f64).powi(2) +
            (bot_pos.y - self.origin.1 as f64).powi(2) +
            (bot_pos.z - self.origin.2 as f64).powi(2)
        ).sqrt();

        if dist > 10.0 {
            tracing::info!("Teleporting to build origin ({}, {}, {})", self.origin.0, self.origin.1, self.origin.2);
            self.bot.chat(&format!(
                "/tp {} {} {}",
                self.origin.0, self.origin.1, self.origin.2
            ));
            self.bot.wait_updates(10).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
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
