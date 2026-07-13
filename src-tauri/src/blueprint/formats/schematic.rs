use std::io::Read;
use flate2::read::GzDecoder;
use fastnbt::{Value, from_value};
use serde::Deserialize;

use crate::blueprint::types::{Blueprint, BlockPalette};

#[derive(Debug, Deserialize)]
struct MCEditSchematic {
    #[serde(rename = "Width")]
    width: Option<i16>,
    #[serde(rename = "Height")]
    height: Option<i16>,
    #[serde(rename = "Length")]
    length: Option<i16>,
    #[serde(rename = "Blocks")]
    blocks: Option<Vec<i8>>,
    #[serde(rename = "Data")]
    data: Option<Vec<i8>>,
    #[serde(rename = "Entities")]
    entities: Option<Vec<Value>>,
    #[serde(rename = "TileEntities")]
    tile_entities: Option<Vec<Value>>,
}

const BLOCK_NAMES: &[&str] = &[
    "minecraft:air", "minecraft:stone", "minecraft:grass_block", "minecraft:dirt",
    "minecraft:cobblestone", "minecraft:oak_planks", "minecraft:oak_sapling",
    "minecraft:bedrock", "minecraft:water", "minecraft:lava", "minecraft:sand",
    "minecraft:gravel", "minecraft:gold_ore", "minecraft:iron_ore", "minecraft:coal_ore",
    "minecraft:oak_log", "minecraft:oak_leaves", "minecraft:sponge", "minecraft:glass",
    "minecraft:lapis_ore", "minecraft:lapis_block", "minecraft:dispenser",
    "minecraft:sandstone", "minecraft:note_block", "minecraft:bed", "minecraft:golden_rail",
    "minecraft:detector_rail", "minecraft:sticky_piston", "minecraft:cobweb", "minecraft:tall_grass",
    "minecraft:dead_bush", "minecraft:piston", "minecraft:piston_head",
    "minecraft:wool", "minecraft:gold_block", "minecraft:iron_block",
    "minecraft:brick_block", "minecraft:tnt", "minecraft:bookshelf",
    "minecraft:mossy_cobblestone", "minecraft:obsidian", "minecraft:torch",
    "minecraft:fire", "minecraft:mob_spawner", "minecraft:oak_stairs",
    "minecraft:chest", "minecraft:redstone_wire", "minecraft:diamond_ore",
    "minecraft:diamond_block", "minecraft:crafting_table", "minecraft:wheat",
    "minecraft:farmland", "minecraft:furnace", "minecraft:lit_furnace",
];

pub fn parse_schematic(data: &[u8]) -> Result<Blueprint, String> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| format!("Failed to decompress .schematic: {}", e))?;

    let value: Value = fastnbt::from_bytes(&decompressed)
        .map_err(|e| format!("Failed to parse NBT: {}", e))?;

    let schematic: MCEditSchematic = from_value(&value)
        .map_err(|e| format!("Failed to deserialize .schematic: {}", e))?;

    let width = schematic.width.unwrap_or(0) as u32;
    let height = schematic.height.unwrap_or(0) as u32;
    let length = schematic.length.unwrap_or(0) as u32;

    let blocks_data = schematic.blocks
        .ok_or("No blocks data in .schematic")?;

    let mut block_palette = BlockPalette::new();
    let mut blocks = Vec::with_capacity(height as usize);

    let _blocks_per_layer = (width * length) as usize;
    let mut idx = 0;

    for _y in 0..height {
        let mut layer = Vec::with_capacity(length as usize);
        for _z in 0..length {
            let mut row = Vec::with_capacity(width as usize);
            for _x in 0..width {
                if idx < blocks_data.len() {
                    let block_id = blocks_data[idx] as usize;
                    let block_name = if block_id < BLOCK_NAMES.len() {
                        BLOCK_NAMES[block_id]
                    } else {
                        "minecraft:air"
                    };

                    if block_name != "minecraft:air" {
                        block_palette.add_symbol(block_name.to_string(), block_name.to_string());
                        row.push(Some(block_name.to_string()));
                    } else {
                        row.push(None);
                    }
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
        source: Some("schematic".to_string()),
        width,
        height,
        length,
        palette: block_palette,
        blocks,
        materials: None,
        description: Some("MCEdit Schematic format".to_string()),
    })
}
