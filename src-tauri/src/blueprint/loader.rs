use std::path::Path;
use super::types::{Blueprint, BlockPlacement};
use super::parser::parse_blueprint;
use super::importers::GrabCraftImporter;
use super::formats::{litematic, schem, schematic, nbt_structure};

pub struct BlueprintLoader;

impl BlueprintLoader {
    pub fn load_from_file(path: &Path) -> Result<Blueprint, String> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        match ext.as_str() {
            "json" | "blueprint" => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                if ext == "json" {
                    serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
                } else {
                    parse_blueprint(&content)
                }
            }
            "mcfunction" => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                parse_mcfunction(&content, path)
            }
            "litematic" => {
                let data = std::fs::read(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                litematic::parse_litematic(&data)
            }
            "schem" => {
                let data = std::fs::read(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                schem::parse_schem(&data)
            }
            "schematic" => {
                let data = std::fs::read(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                schematic::parse_schematic(&data)
            }
            "nbt" | "mcstructure" => {
                let data = std::fs::read(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                nbt_structure::parse_nbt_structure(&data)
            }
            _ => Err(format!("Unsupported file format: .{}", ext)),
        }
    }

    pub async fn load_from_url(url: &str) -> Result<Blueprint, String> {
        if url.contains("grabcraft.com") {
            let importer = GrabCraftImporter::new();
            importer.import(url).await
        } else {
            Err(format!("Unsupported URL: {}", url))
        }
    }

    pub fn save_to_file(blueprint: &Blueprint, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(blueprint)
            .map_err(|e| format!("Failed to serialize blueprint: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
    }
}

fn parse_mcfunction(content: &str, path: &Path) -> Result<Blueprint, String> {
    let mut placements = Vec::new();
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut min_z = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut max_z = i32::MIN;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(pos) = parse_setblock_command(line) {
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            min_z = min_z.min(pos.z);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
            max_z = max_z.max(pos.z);
            placements.push(pos);
        } else if let Some(fills) = parse_fill_command(line) {
            for pos in fills {
                min_x = min_x.min(pos.x);
                min_y = min_y.min(pos.y);
                min_z = min_z.min(pos.z);
                max_x = max_x.max(pos.x);
                max_y = max_y.max(pos.y);
                max_z = max_z.max(pos.z);
                placements.push(pos);
            }
        }
    }

    if placements.is_empty() {
        return Err("No valid setblock/fill commands found".to_string());
    }

    let width = (max_x - min_x + 1) as u32;
    let height = (max_y - min_y + 1) as u32;
    let length = (max_z - min_z + 1) as u32;

    let mut blocks = vec![vec![vec![None; width as usize]; length as usize]; height as usize];
    let mut palette = super::types::BlockPalette::new();

    for placement in &placements {
        let x = (placement.x - min_x) as usize;
        let y = (placement.y - min_y) as usize;
        let z = (placement.z - min_z) as usize;

        if y < blocks.len() && z < blocks[y].len() && x < blocks[y][z].len() {
            blocks[y][z][x] = Some(placement.block_id.clone());
            palette.add_symbol(placement.block_id.clone(), placement.block_id.clone());
        }
    }

    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("mcfunction_blueprint")
        .to_string();

    Ok(Blueprint {
        name,
        author: None,
        source: None,
        width,
        height,
        length,
        palette,
        blocks,
        materials: None,
        description: Some(format!("Parsed from .mcfunction file")),
    })
}

fn parse_setblock_command(line: &str) -> Option<BlockPlacement> {
    let line = line.trim();
    if !line.starts_with("setblock ") {
        return None;
    }

    let args: Vec<&str> = line[9..].split_whitespace().collect();
    if args.len() < 4 {
        return None;
    }

    let x = parse_coord(args[0])?;
    let y = parse_coord(args[1])?;
    let z = parse_coord(args[2])?;
    let block = args[3..].join(" ");

    Some(BlockPlacement::new(x, y, z, block))
}

fn parse_fill_command(line: &str) -> Option<Vec<BlockPlacement>> {
    let line = line.trim();
    if !line.starts_with("fill ") {
        return None;
    }

    let args: Vec<&str> = line[5..].split_whitespace().collect();
    if args.len() < 7 {
        return None;
    }

    let x1 = parse_coord(args[0])?;
    let y1 = parse_coord(args[1])?;
    let z1 = parse_coord(args[2])?;
    let x2 = parse_coord(args[3])?;
    let y2 = parse_coord(args[4])?;
    let z2 = parse_coord(args[5])?;
    let block = args[6..].join(" ");

    let mut placements = Vec::new();
    for x in x1..=x2 {
        for y in y1..=y2 {
            for z in z1..=z2 {
                placements.push(BlockPlacement::new(x, y, z, block.clone()));
            }
        }
    }

    Some(placements)
}

fn parse_coord(s: &str) -> Option<i32> {
    s.parse().ok()
}
