use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub bot: BotConfig,
    pub ai: AiConfig,
    pub automation: AutomationConfig,
    pub starter_kit: Vec<KitItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub auto_reconnect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub username: String,
    pub permission_mode: PermissionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionMode {
    Player,
    Operator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub auto_sleep: bool,
    pub auto_eat: bool,
    pub auto_respawn: bool,
    pub auto_reconnect: bool,
    pub welcome_messages: bool,
    pub starter_kit_on_respawn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitItem {
    pub item: String,
    pub count: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: "localhost".to_string(),
                port: 25565,
                auto_reconnect: true,
            },
            bot: BotConfig {
                username: "MineMate".to_string(),
                permission_mode: PermissionMode::Player,
            },
            ai: AiConfig {
                api_key: String::new(),
                model: "meta/llama-3.3-70b-instruct".to_string(),
                temperature: 0.7,
                max_tokens: 1024,
            },
            automation: AutomationConfig {
                auto_sleep: true,
                auto_eat: true,
                auto_respawn: true,
                auto_reconnect: true,
                welcome_messages: true,
                starter_kit_on_respawn: true,
            },
            starter_kit: vec![
                KitItem {
                    item: "diamond_sword".to_string(),
                    count: 1,
                },
                KitItem {
                    item: "diamond_pickaxe".to_string(),
                    count: 1,
                },
                KitItem {
                    item: "diamond_axe".to_string(),
                    count: 1,
                },
                KitItem {
                    item: "cooked_beef".to_string(),
                    count: 64,
                },
            ],
        }
    }
}

impl AppConfig {
    fn config_path() -> std::path::PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("MineMate").join("config").join("default.toml")
        } else {
            let dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            dir.join("config").join("default.toml")
        }
    }

    fn legacy_config_path() -> std::path::PathBuf {
        let dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        dir.join("config").join("default.toml")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        let legacy_path = Self::legacy_config_path();

        // Migrate from legacy path if it exists and new path doesn't
        if !config_path.exists() && legacy_path.exists() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if let Err(e) = std::fs::copy(&legacy_path, &config_path) {
                tracing::warn!("Failed to migrate config from {:?}: {}", legacy_path, e);
            }
        }

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            let config = Self::default();
            let content = toml::to_string_pretty(&config)?;
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&config_path, content)?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        std::fs::create_dir_all(config_path.parent().unwrap())?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
