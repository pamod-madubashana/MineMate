use azalea::Client;
use crate::blueprint::types::Blueprint;
use crate::blueprint::materials::{check_materials, MaterialList};
use super::planner::plan_build;
use super::placement::place_blocks;

pub struct BuildExecutor {
    bot: Client,
    blueprint: Blueprint,
    origin: (i32, i32, i32),
    inventory: std::collections::HashMap<String, u32>,
}

impl BuildExecutor {
    pub fn new(bot: Client, blueprint: Blueprint, origin: (i32, i32, i32)) -> Self {
        Self { bot, blueprint, origin, inventory: std::collections::HashMap::new() }
    }

    pub fn set_inventory(&mut self, inventory: std::collections::HashMap<String, u32>) {
        self.inventory = inventory;
    }

    pub fn check_materials(&self) -> MaterialList {
        check_materials(&self.blueprint, &self.inventory)
    }

    pub async fn execute(&self) -> Result<u32, String> {
        let plan = plan_build(&self.blueprint, self.origin);
        tracing::info!("Starting build: {} blocks, {} layers", plan.total_blocks, plan.layers);

        let mut total_placed = 0;
        let mut current_y = i32::MIN;

        for chunk in plan.placements.chunks(100) {
            if let Some(first) = chunk.first() {
                if first.y != current_y {
                    current_y = first.y;
                    tracing::info!("Building layer y={}", current_y);
                }
            }

            match place_blocks(&self.bot, chunk).await {
                Ok(placed) => { total_placed += placed; }
                Err(e) => {
                    tracing::error!("Build error: {}", e);
                    return Err(e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        tracing::info!("Build complete: {}/{} blocks placed", total_placed, plan.total_blocks);
        Ok(total_placed)
    }
}
