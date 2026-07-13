use std::io::Cursor;
use image::GenericImageView;
use super::legend::GrabCraftLegend;

pub fn parse_layer_image(
    image_data: &[u8],
    legend: &GrabCraftLegend,
    layer_y: u32,
) -> Result<Vec<Vec<Option<String>>>, String> {
    let img = image::load(Cursor::new(image_data), image::ImageFormat::Png)
        .or_else(|_| image::load(Cursor::new(image_data), image::ImageFormat::Jpeg))
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    let mut layer = Vec::new();

    for z in 0..height {
        let mut row = Vec::new();

        for x in 0..width {
            let pixel = rgba.get_pixel(x, z);
            let [r, g, b, a] = pixel.0;

            if a < 128 {
                row.push(None);
                continue;
            }

            if let Some(block_id) = legend.match_color(r, g, b) {
                row.push(Some(block_id.to_string()));
            } else {
                let closest = find_closest_color(legend, r, g, b);
                row.push(closest);
            }
        }

        layer.push(row);
    }

    Ok(layer)
}

fn find_closest_color(legend: &GrabCraftLegend, r: u8, g: u8, b: u8) -> Option<String> {
    let mut best_match = None;
    let mut best_distance = f64::MAX;

    for (color, block_id) in &legend.color_to_block {
        let dr = r as f64 - color[0] as f64;
        let dg = g as f64 - color[1] as f64;
        let db = b as f64 - color[2] as f64;
        let distance = (dr * dr + dg * dg + db * db).sqrt();

        if distance < best_distance {
            best_distance = distance;
            best_match = Some(block_id.clone());
        }
    }

    if best_distance < 50.0 {
        best_match
    } else {
        None
    }
}
