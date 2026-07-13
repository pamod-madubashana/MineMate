use std::collections::HashMap;
use std::io::Read;
use flate2::read::GzDecoder;
use fastnbt::{Value, from_value};
use serde::Deserialize;

use crate::blueprint::types::{Blueprint, BlockPalette};

#[derive(Debug, Deserialize)]
struct Structure {
    #[serde(rename = "size")]
    size: Option<Vec<i32>>,
    #[serde(rename = "blocks")]
    blocks: Option<Vec<StructureBlock>>,
    #[serde(rename = "entities")]
    entities: Option<Vec<Value>>,
    #[serde(rename = "data_version")]
    data_version: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct StructureBlock {
    #[serde(rename = "state")]
    state: Option<i32>,
    #[serde(rename = "pos")]
    pos: Option<Vec<i32>>,
    #[serde(rename = "nbt")]
    nbt: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct StatePalette {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties")]
    properties: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct StructureWithPalette {
    #[serde(rename = "size")]
    size: Option<Vec<i32>>,
    #[serde(rename = "blocks")]
    blocks: Option<Vec<StructureBlock>>,
    #[serde(rename = "palette")]
    palette: Option<Vec<StatePalette>>,
    #[serde(rename = "entities")]
    entities: Option<Vec<Value>>,
    #[serde(rename = "data_version")]
    data_version: Option<i32>,
}

pub fn parse_nbt_structure(data: &[u8]) -> Result<Blueprint, String> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| format!("Failed to decompress structure: {}", e))?;

    let value: Value = fastnbt::from_bytes(&decompressed)
        .map_err(|e| format!("Failed to parse NBT: {}", e))?;

    let structure: StructureWithPalette = from_value(&value)
        .map_err(|e| format!("Failed to deserialize structure: {}", e))?;

    let size = structure.size
        .ok_or("No size in structure")?;

    let width = size[0] as u32;
    let height = size[1] as u32;
    let length = size[2] as u32;

    let blocks = structure.blocks
        .ok_or("No blocks in structure")?;

    let palette = structure.palette
        .ok_or("No palette in structure")?;

    let mut block_palette = BlockPalette::new();
    let mut block_grid = vec![vec![vec![None; width as usize]; length as usize]; height as usize];

    for block in &blocks {
        if let (Some(pos), Some(state)) = (&block.pos, block.state) {
            let x = (pos[0] + width as i32 / 2) as usize;
            let y = (pos[1]) as usize;
            let z = (pos[2] + length as i32 / 2) as usize;

            if state >= 0 && (state as usize) < palette.len() {
                let palette_entry = &palette[state as usize];
                let block_name = &palette_entry.name;

                if block_name != "minecraft:air" {
                    let block_id = if let Some(props) = &palette_entry.properties {
                        let props_str: Vec<String> = props.iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect();
                        format!("{}[{}]", block_name, props_str.join(","))
                    } else {
                        block_name.clone()
                    };

                    block_palette.add_symbol(block_id.clone(), block_id.clone());

                    if y < block_grid.len() && z < block_grid[y].len() && x < block_grid[y][z].len() {
                        block_grid[y][z][x] = Some(block_id);
                    }
                }
            }
        }
    }

    Ok(Blueprint {
        name: "structure".to_string(),
        author: None,
        source: Some("nbt_structure".to_string()),
        width,
        height,
        length,
        palette: block_palette,
        blocks: block_grid,
        materials: None,
        description: Some("Minecraft Structure NBT format".to_string()),
    })
}
