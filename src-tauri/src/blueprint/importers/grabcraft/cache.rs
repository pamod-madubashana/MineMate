use std::path::PathBuf;
use crate::blueprint::types::Blueprint;

pub struct BlueprintCache {
    cache_dir: PathBuf,
}

impl BlueprintCache {
    pub fn new() -> Self {
        let cache_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("MineMate")
            .join("cache")
            .join("blueprints");

        Self { cache_dir }
    }

    pub fn get(&self, name: &str) -> Option<Blueprint> {
        let path = self.cache_dir.join(format!("{}.json", name));
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, name: &str, blueprint: &Blueprint) -> Result<(), String> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        let path = self.cache_dir.join(format!("{}.json", name));
        let content = serde_json::to_string_pretty(blueprint)
            .map_err(|e| format!("Failed to serialize blueprint: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write cache file: {}", e))
    }

    pub fn list(&self) -> Vec<String> {
        if !self.cache_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(&self.cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.extension().map(|e| e == "json").unwrap_or(false) {
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn clear(&self) -> Result<(), String> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| format!("Failed to clear cache: {}", e))?;
        }
        Ok(())
    }
}

impl Default for BlueprintCache {
    fn default() -> Self {
        Self::new()
    }
}
