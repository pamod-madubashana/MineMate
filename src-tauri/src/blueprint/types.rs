use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub author: Option<String>,
    pub source: Option<String>,
    pub width: u32,
    pub height: u32,
    pub length: u32,
    pub palette: BlockPalette,
    pub blocks: Vec<Vec<Vec<Option<String>>>>,
    pub materials: Option<MaterialCount>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPalette {
    pub symbols: HashMap<String, String>,
}

impl BlockPalette {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: String, block_id: String) {
        self.symbols.insert(symbol, block_id);
    }

    pub fn get_block(&self, symbol: &str) -> Option<&str> {
        self.symbols.get(symbol).map(|s| s.as_str())
    }

    pub fn block_id(&self, symbol: &str) -> Option<String> {
        self.symbols.get(symbol).cloned()
    }
}

impl Default for BlockPalette {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCount {
    pub materials: HashMap<String, u32>,
}

impl MaterialCount {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    pub fn add(&mut self, block_id: &str, count: u32) {
        *self.materials.entry(block_id.to_string()).or_insert(0) += count;
    }

    pub fn get(&self, block_id: &str) -> u32 {
        self.materials.get(block_id).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u32 {
        self.materials.values().sum()
    }
}

impl Default for MaterialCount {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlacement {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_id: String,
}

impl BlockPlacement {
    pub fn new(x: i32, y: i32, z: i32, block_id: String) -> Self {
        Self { x, y, z, block_id }
    }
}
