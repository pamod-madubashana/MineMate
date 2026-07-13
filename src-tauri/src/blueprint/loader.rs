use std::path::Path;
use super::types::Blueprint;
use super::parser::parse_blueprint;
use super::importers::GrabCraftImporter;

pub struct BlueprintLoader;

impl BlueprintLoader {
    pub fn load_from_file(path: &Path) -> Result<Blueprint, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "json" => serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e)),
            "blueprint" => parse_blueprint(&content),
            _ => Err(format!("Unsupported file format: {}", ext)),
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
