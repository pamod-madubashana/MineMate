use std::collections::HashMap;
use std::io::Read;
use flate2::read::GzDecoder;
use fastnbt::{Value, from_value};
use serde::Deserialize;

use crate::blueprint::types::{Blueprint, BlockPalette};

#[derive(Debug, Deserialize)]
struct Litematic {
    #[serde(rename = "Metadata")]
    metadata: Option<LitematicMetadata>,
    #[serde(rename = "Regions")]
    regions: HashMap<String, Region>,
    #[serde(rename = "Version")]
    version: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct LitematicMetadata {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "Description")]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Region {
    #[serde(rename = "Size")]
    size: Option<Vec<i32>>,
    #[serde(rename = "Position")]
    position: Option<Vec<i32>>,
    #[serde(rename = "BlockStatePalette")]
    palette: Option<Vec<PaletteEntry>>,
    #[serde(rename = "BlockStates")]
    block_states: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
struct PaletteEntry {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties")]
    properties: Option<HashMap<String, String>>,
}

pub fn parse_litematic(data: &[u8]) -> Result<Blueprint, String> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| format!("Failed to decompress litematic: {}", e))?;

    let value: Value = fastnbt::from_bytes(&decompressed)
        .map_err(|e| format!("Failed to parse NBT: {}", e))?;

    let litematic: Litematic = from_value(&value)
        .map_err(|e| format!("Failed to deserialize litematic: {}", e))?;

    let region_name = litematic.regions.keys().next()
        .ok_or("No regions found in litematic")?
        .clone();

    let region = litematic.regions.get(&region_name)
        .ok_or("Region not found")?;

    let size = region.size.as_ref()
        .ok_or("No size in region")?;

    let width = size[0].unsigned_abs() as u32;
    let height = size[1].unsigned_abs() as u32;
    let length = size[2].unsigned_abs() as u32;

    let palette = region.palette.as_ref()
        .ok_or("No palette in region")?;

    let block_states = region.block_states.as_ref()
        .ok_or("No block states in region")?;

    let palette_map: Vec<String> = palette.iter().map(|entry| {
        if let Some(props) = &entry.properties {
            let props_str: Vec<String> = props.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("{}[{}]", entry.name, props_str.join(","))
        } else {
            entry.name.clone()
        }
    }).collect();

    let blocks_per_layer = (width * length) as usize;
    let total_blocks = (width * height * length) as usize;

    let mut blocks = Vec::with_capacity(height as usize);
    let mut block_palette = BlockPalette::new();

    for y in 0..height as usize {
        let mut layer = Vec::with_capacity(length as usize);
        for z in 0..length as usize {
            let mut row = Vec::with_capacity(width as usize);
            for x in 0..width as usize {
                let idx = y * blocks_per_layer + z * width as usize + x;
                let block_id = if idx < block_states.len() {
                    let palette_idx = block_states[idx] as usize;
                    if palette_idx < palette_map.len() {
                        Some(palette_map[palette_idx].clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(ref id) = block_id {
                    block_palette.add_symbol(id.clone(), id.clone());
                }

                row.push(block_id);
            }
            layer.push(row);
        }
        blocks.push(layer);
    }

    let name = litematic.metadata.as_ref()
        .and_then(|m| m.name.clone())
        .unwrap_or_else(|| region_name.clone());

    let author = litematic.metadata.as_ref()
        .and_then(|m| m.author.clone());

    let description = litematic.metadata.as_ref()
        .and_then(|m| m.description.clone());

    Ok(Blueprint {
        name,
        author,
        source: Some("litematic".to_string()),
        width,
        height,
        length,
        palette: block_palette,
        blocks,
        materials: None,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_litematic() {
        let result = parse_litematic(&[]);
        assert!(result.is_err());
    }
}
