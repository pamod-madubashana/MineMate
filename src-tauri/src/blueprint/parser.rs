use super::types::{Blueprint, BlockPalette, MaterialCount};

pub fn parse_blueprint(content: &str) -> Result<Blueprint, String> {
    let json: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let name = json["name"].as_str().unwrap_or("Unnamed Blueprint").to_string();
    let author = json["author"].as_str().map(|s| s.to_string());
    let source = json["source"].as_str().map(|s| s.to_string());
    let description = json["description"].as_str().map(|s| s.to_string());
    let width = json["width"].as_u64().unwrap_or(1) as u32;
    let height = json["height"].as_u64().unwrap_or(1) as u32;
    let length = json["length"].as_u64().unwrap_or(1) as u32;

    let palette = parse_palette(&json["palette"])?;
    let blocks = parse_blocks(&json["blocks"], &palette, width, height, length)?;
    let materials = parse_materials(&json["materials"]);

    Ok(Blueprint { name, author, source, width, height, length, palette, blocks, materials, description })
}

fn parse_palette(json: &serde_json::Value) -> Result<BlockPalette, String> {
    let mut palette = BlockPalette::new();
    if let Some(obj) = json.as_object() {
        for (symbol, block_id) in obj {
            if let Some(id) = block_id.as_str() {
                palette.add_symbol(symbol.clone(), id.to_string());
            }
        }
    }
    Ok(palette)
}

fn parse_blocks(json: &serde_json::Value, palette: &BlockPalette, width: u32, height: u32, length: u32) -> Result<Vec<Vec<Vec<Option<String>>>>, String> {
    let mut blocks = Vec::new();
    if let Some(layers) = json.as_array() {
        for (y, layer) in layers.iter().enumerate() {
            if y >= height as usize { break; }
            let mut y_layer = Vec::new();
            if let Some(rows) = layer.as_array() {
                for (z, row) in rows.iter().enumerate() {
                    if z >= length as usize { break; }
                    let mut z_row = Vec::new();
                    if let Some(cells) = row.as_array() {
                        for (x, cell) in cells.iter().enumerate() {
                            if x >= width as usize { break; }
                            let symbol = cell.as_str().unwrap_or("");
                            let block_id = if symbol.is_empty() || symbol == " " { None } else { palette.block_id(symbol) };
                            z_row.push(block_id);
                        }
                    }
                    while z_row.len() < width as usize { z_row.push(None); }
                    y_layer.push(z_row);
                }
            }
            while y_layer.len() < length as usize {
                let mut empty_row = Vec::new();
                empty_row.resize(width as usize, None);
                y_layer.push(empty_row);
            }
            blocks.push(y_layer);
        }
    }
    while blocks.len() < height as usize {
        let mut empty_layer = Vec::new();
        for _ in 0..length {
            let mut empty_row = Vec::new();
            empty_row.resize(width as usize, None);
            empty_layer.push(empty_row);
        }
        blocks.push(empty_layer);
    }
    Ok(blocks)
}

fn parse_materials(json: &serde_json::Value) -> Option<MaterialCount> {
    let obj = json.as_object()?;
    let mut materials = MaterialCount::new();
    for (block_id, count) in obj {
        if let Some(count) = count.as_u64() {
            materials.add(block_id, count as u32);
        }
    }
    Some(materials)
}
