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

        let mut placer = PlayerPlacer::new(self.bot.clone());
        placer.prepare_for_building().await?;

        let total_placed;

        match placer.place_blocks(&plan.placements).await {
            Ok(placed) => { total_placed = placed; }
            Err(e) => {
                tracing::error!("Build error: {}", e);
                return Err(e);
            }
        }

        tracing::info!("Build complete: {}/{} blocks placed", total_placed, plan.total_blocks);
        Ok(total_placed)
    }
}
