use crate::blueprint::types::{Blueprint, BlockPlacement};

#[derive(Debug, Clone)]
pub struct BuildPlan {
    pub placements: Vec<BlockPlacement>,
    pub origin: (i32, i32, i32),
    pub total_blocks: u32,
    pub layers: u32,
}

impl BuildPlan {
    pub fn new(origin: (i32, i32, i32)) -> Self {
        Self { placements: Vec::new(), origin, total_blocks: 0, layers: 0 }
    }

    pub fn sort_bottom_up(&mut self) {
        self.placements.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)).then(a.z.cmp(&b.z)));
    }

    pub fn get_layer_count(&self) -> i32 {
        if self.placements.is_empty() { return 0; }
        let min_y = self.placements.iter().map(|p| p.y).min().unwrap_or(0);
        let max_y = self.placements.iter().map(|p| p.y).max().unwrap_or(0);
        max_y - min_y + 1
    }
}

pub fn plan_build(blueprint: &Blueprint, origin: (i32, i32, i32)) -> BuildPlan {
    let mut plan = BuildPlan::new(origin);
    for (y, layer) in blueprint.blocks.iter().enumerate() {
        for (z, row) in layer.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Some(block_id) = cell {
                    let placement = BlockPlacement::new(
                        origin.0 + x as i32,
                        origin.1 + y as i32,
                        origin.2 + z as i32,
                        block_id.clone(),
                    );
                    plan.placements.push(placement);
                    plan.total_blocks += 1;
                }
            }
        }
    }
    plan.layers = plan.get_layer_count() as u32;
    plan.sort_bottom_up();
    plan
}
