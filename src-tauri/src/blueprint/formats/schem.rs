use std::collections::HashMap;
use std::io::Read;
use flate2::read::GzDecoder;
use fastnbt::{Value, from_value};
use serde::Deserialize;

use crate::blueprint::types::{Blueprint, BlockPalette};

#[derive(Debug, Deserialize)]
struct SpongeSchematic {
    #[serde(rename = "Version")]
    version: Option<i32>,
    #[serde(rename = "DataVersion")]
    data_version: Option<i32>,
    #[serde(rename = "Width")]
    width: Option<i16>,
    #[serde(rename = "Height")]
    height: Option<i16>,
    #[serde(rename = "Length")]
    length: Option<i16>,
    #[serde(rename = "Palette")]
    palette: Option<HashMap<String, i32>>,
    #[serde(rename = "PaletteMax")]
    palette_max: Option<i32>,
    #[serde(rename = "BlockData")]
    block_data: Option<Vec<i8>>,
}

pub fn parse_schem(data: &[u8]) -> Result<Blueprint, String> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| format!("Failed to decompress .schem: {}", e))?;

    let value: Value = fastnbt::from_bytes(&decompressed)
        .map_err(|e| format!("Failed to parse NBT: {}", e))?;

    let schematic: SpongeSchematic = from_value(&value)
        .map_err(|e| format!("Failed to deserialize .schem: {}", e))?;

    let width = schematic.width.unwrap_or(0) as u32;
    let height = schematic.height.unwrap_or(0) as u32;
    let length = schematic.length.unwrap_or(0) as u32;

    let palette = schematic.palette
        .ok_or("No palette in .schem")?;

    let block_data = schematic.block_data
        .ok_or("No block data in .schem")?;

    let palette_map: HashMap<i32, String> = palette.into_iter()
        .map(|(name, id)| (id, name))
        .collect();

    let mut block_palette = BlockPalette::new();
    let mut blocks = Vec::with_capacity(height as usize);

    let blocks_per_layer = (width * length) as usize;
    let mut idx = 0;

    for _y in 0..height {
        let mut layer = Vec::with_capacity(length as usize);
        for _z in 0..length {
            let mut row = Vec::with_capacity(width as usize);
            for _x in 0..width {
                if idx < block_data.len() {
                    let palette_id = block_data[idx] as i32;
                    let block_id = palette_map.get(&palette_id).cloned();
                    if let Some(ref id) = block_id {
                        block_palette.add_symbol(id.clone(), id.clone());
                    }
                    row.push(block_id);
                } else {
                    row.push(None);
                }
                idx += 1;
            }
            layer.push(row);
        }
        blocks.push(layer);
    }

    Ok(Blueprint {
        name: "schematic".to_string(),
        author: None,
        source: Some("schem".to_string()),
        width,
        height,
        length,
        palette: block_palette,
        blocks,
        materials: None,
        description: Some("Sponge Schematic format".to_string()),
    })
}
