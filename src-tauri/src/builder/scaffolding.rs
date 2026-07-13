use crate::blueprint::types::{Blueprint, BlockPlacement};

pub struct ScaffoldPlanner;

impl ScaffoldPlanner {
    pub fn needs_scaffolding(blueprint: &Blueprint) -> bool {
        blueprint.height > 4
    }

    pub fn plan_scaffolding(blueprint: &Blueprint, origin: (i32, i32, i32), scaffold_block: &str) -> Vec<BlockPlacement> {
        let mut placements = Vec::new();
        if !Self::needs_scaffolding(blueprint) { return placements; }

        let width = blueprint.width as i32;
        let length = blueprint.length as i32;
        let height = blueprint.height as i32;

        for y in (4..height).step_by(4) {
            for x in 0..width {
                for z in 0..length {
                    let check_y = origin.1 + y;
                    let block_below = blueprint.blocks
                        .get((y - 1) as usize)
                        .and_then(|l| l.get(z as usize))
                        .and_then(|r| r.get(x as usize))
                        .and_then(|b| b.as_ref());

                    if block_below.is_none() {
                        placements.push(BlockPlacement::new(origin.0 + x, check_y, origin.2 + z, scaffold_block.to_string()));
                    }
                }
            }
        }

        placements
    }
}
