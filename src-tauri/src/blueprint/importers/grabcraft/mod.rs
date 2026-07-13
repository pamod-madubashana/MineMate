pub mod cache;
pub mod downloader;
pub mod html_parser;
pub mod image_parser;
pub mod legend;

use std::path::PathBuf;

use crate::blueprint::types::Blueprint;

use self::cache::BlueprintCache;
use self::downloader::GrabCraftDownloader;
use self::html_parser::parse_grabcraft_page;
use self::image_parser::parse_layer_image;
use self::legend::GrabCraftLegend;

pub struct GrabCraftImporter {
    downloader: GrabCraftDownloader,
    cache: BlueprintCache,
}

impl GrabCraftImporter {
    pub fn new() -> Self {
        Self {
            downloader: GrabCraftDownloader::new(),
            cache: BlueprintCache::new(),
        }
    }

    pub async fn import(&self, url: &str) -> Result<Blueprint, String> {
        let name = self.extract_name(url);

        if let Some(blueprint) = self.cache.get(&name) {
            tracing::info!("Loaded blueprint from cache: {}", name);
            return Ok(blueprint);
        }

        tracing::info!("Importing GrabCraft blueprint: {}", url);

        let html = self.downloader.fetch_page(url).await?;
        let page_data = parse_grabcraft_page(&html)?;

        let legend = GrabCraftLegend::from_html(&html)?;

        let mut layers = Vec::new();

        for (y, layer_url) in page_data.layer_urls.iter().enumerate() {
            tracing::info!("Downloading layer {}/{}", y + 1, page_data.layer_urls.len());

            let image_data = self.downloader.download_image(layer_url).await?;
            let layer = parse_layer_image(&image_data, &legend, y as u32)?;
            layers.push(layer);
        }

        let width = layers.first().map(|l| l.first().map(|r| r.len()).unwrap_or(0)).unwrap_or(0) as u32;
        let height = layers.len() as u32;
        let length = layers.first().map(|l| l.len()).unwrap_or(0) as u32;

        let mut palette = crate::blueprint::types::BlockPalette::new();
        for layer in &layers {
            for row in layer {
                for cell in row {
                    if let Some(block_id) = cell {
                        if !palette.symbols.contains_key(block_id) {
                            palette.add_symbol(block_id.clone(), block_id.clone());
                        }
                    }
                }
            }
        }

        let blueprint = Blueprint {
            name: page_data.name.unwrap_or(name.clone()),
            author: page_data.author,
            source: Some(url.to_string()),
            width,
            height,
            length,
            palette,
            blocks: layers,
            materials: None,
            description: page_data.description,
        };

        self.cache.save(&name, &blueprint)?;

        tracing::info!("Imported blueprint: {} ({}x{}x{})", blueprint.name, blueprint.width, blueprint.height, blueprint.length);

        Ok(blueprint)
    }

    fn extract_name(&self, url: &str) -> String {
        url.split('/')
            .last()
            .unwrap_or("unknown")
            .split('?')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

impl Default for GrabCraftImporter {
    fn default() -> Self {
        Self::new()
    }
}
