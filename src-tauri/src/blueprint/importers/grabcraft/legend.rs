use std::collections::HashMap;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct GrabCraftLegend {
    pub color_to_block: HashMap<[u8; 3], String>,
    pub symbol_to_block: HashMap<String, String>,
}

impl GrabCraftLegend {
    pub fn new() -> Self {
        Self {
            color_to_block: HashMap::new(),
            symbol_to_block: HashMap::new(),
        }
    }

    pub fn from_html(html: &str) -> Result<Self, String> {
        let mut legend = Self::new();
        let document = Html::parse_document(html);

        let selectors = [
            ".legend",
            "[class*='legend']",
            ".material-list",
            "[class*='material']",
            ".block-list",
            "[class*='block-list']",
            "table",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    legend.parse_legend_element(&element);
                }
            }
        }

        if legend.color_to_block.is_empty() && legend.symbol_to_block.is_empty() {
            legend.add_default_mappings();
        }

        Ok(legend)
    }

    fn parse_legend_element(&mut self, element: &scraper::ElementRef) {
        let row_selector = Selector::parse("tr, li, .item, [class*='item']").unwrap();
        let cell_selector = Selector::parse("td, span, .color, .symbol, .name").unwrap();

        let rows = element.select(&row_selector);
        for row in rows {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            if cells.len() >= 2 {
                let color_cell = &cells[0];
                let name_cell = &cells[1];

                if let Some(color) = self.extract_color(color_cell) {
                    let name = name_cell.text().collect::<String>().trim().to_string();
                    if !name.is_empty() {
                        let block_id = self.name_to_block_id(&name);
                        self.color_to_block.insert(color, block_id);
                    }
                }

                if let Some(symbol) = self.extract_symbol(color_cell) {
                    let name = name_cell.text().collect::<String>().trim().to_string();
                    if !name.is_empty() {
                        let block_id = self.name_to_block_id(&name);
                        self.symbol_to_block.insert(symbol, block_id);
                    }
                }
            }
        }
    }

    fn extract_color(&self, element: &scraper::ElementRef) -> Option<[u8; 3]> {
        let style = element.value().attr("style")?;
        if let Some(color_str) = style.find("background-color:") {
            let rest = &style[color_str + 17..];
            let hex = rest.trim_start().trim_end_matches(';').trim_start_matches('#');
            if hex.len() >= 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some([r, g, b]);
            }
        }

        if let Some(bg) = element.value().attr("bgcolor") {
            let hex = bg.trim_start_matches('#');
            if hex.len() >= 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some([r, g, b]);
            }
        }

        None
    }

    fn extract_symbol(&self, element: &scraper::ElementRef) -> Option<String> {
        let text = element.text().collect::<String>().trim().to_string();
        if text.len() == 1 && text.chars().next().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false) {
            Some(text)
        } else {
            None
        }
    }

    fn name_to_block_id(&self, name: &str) -> String {
        let lowercase = name.to_lowercase().replace(' ', "_");
        format!("minecraft:{}", lowercase)
    }

    pub fn match_color(&self, r: u8, g: u8, b: u8) -> Option<&str> {
        self.color_to_block.get(&[r, g, b]).map(|s| s.as_str())
    }

    pub fn match_symbol(&self, symbol: &str) -> Option<&str> {
        self.symbol_to_block.get(symbol).map(|s| s.as_str())
    }

    fn add_default_mappings(&mut self) {
        let defaults = [
            ([128, 128, 128], "minecraft:stone"),
            ([139, 69, 19], "minecraft:dirt"),
            ([105, 105, 105], "minecraft:cobblestone"),
            ([160, 82, 45], "minecraft:oak_planks"),
            ([200, 200, 200], "minecraft:glass"),
            ([0, 128, 0], "minecraft:grass_block"),
            ([0, 0, 0], "minecraft:black_wool"),
            ([255, 255, 255], "minecraft:white_wool"),
            ([255, 0, 0], "minecraft:red_wool"),
            ([0, 255, 0], "minecraft:green_wool"),
            ([0, 0, 255], "minecraft:blue_wool"),
            ([255, 255, 0], "minecraft:yellow_wool"),
        ];

        for (color, block) in defaults {
            self.color_to_block.insert(color, block.to_string());
        }
    }
}

impl Default for GrabCraftLegend {
    fn default() -> Self {
        Self::new()
    }
}
