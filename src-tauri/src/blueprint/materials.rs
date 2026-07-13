use std::collections::HashMap;
use super::types::{Blueprint, MaterialCount};

#[derive(Debug, Clone)]
pub struct MaterialList {
    pub required: MaterialCount,
    pub available: MaterialCount,
    pub missing: MaterialCount,
}

impl MaterialList {
    pub fn new() -> Self {
        Self { required: MaterialCount::new(), available: MaterialCount::new(), missing: MaterialCount::new() }
    }

    pub fn is_complete(&self) -> bool {
        self.missing.total() == 0
    }

    pub fn completion_percentage(&self) -> f32 {
        let total = self.required.total();
        if total == 0 { return 100.0; }
        let have = self.available.total().min(total);
        (have as f32 / total as f32) * 100.0
    }
}

impl Default for MaterialList {
    fn default() -> Self { Self::new() }
}

pub fn estimate_materials(blueprint: &Blueprint) -> MaterialList {
    let mut required = MaterialCount::new();
    for layer in &blueprint.blocks {
        for row in layer {
            for cell in row {
                if let Some(block_id) = cell {
                    required.add(block_id, 1);
                }
            }
        }
    }
    let mut list = MaterialList::new();
    list.required = required;
    list
}

pub fn check_materials(blueprint: &Blueprint, inventory: &HashMap<String, u32>) -> MaterialList {
    let mut list = estimate_materials(blueprint);
    for (block_id, needed) in &list.required.materials {
        let have = inventory.get(block_id).copied().unwrap_or(0);
        list.available.add(block_id, have);
        if have < *needed {
            list.missing.add(block_id, needed - have);
        }
    }
    list
}
