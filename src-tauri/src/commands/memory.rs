use crate::memory::db::{Database, Player, Location, HistoryEntry, Blueprint};
use std::sync::OnceLock;

static DB: OnceLock<Database> = OnceLock::new();

fn get_db() -> &'static Database {
    DB.get_or_init(|| Database::new("database/minemate.db").expect("Failed to initialize database"))
}

#[tauri::command]
pub async fn list_players() -> Result<Vec<Player>, String> {
    get_db().list_players().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_player(name: String) -> Result<Player, String> {
    get_db().save_player(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_locations() -> Result<Vec<Location>, String> {
    get_db().list_locations().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_location(name: String, x: i32, y: i32, z: i32, dimension: String, description: String) -> Result<Location, String> {
    get_db().save_location(&name, x, y, z, &dimension, &description).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_blueprints() -> Result<Vec<Blueprint>, String> {
    get_db().list_blueprints().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_blueprint(name: String, data: String, author: String) -> Result<Blueprint, String> {
    get_db().save_blueprint(&name, &data, &author).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(limit: u32) -> Result<Vec<HistoryEntry>, String> {
    get_db().get_history(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn log_event(event_type: String, player: Option<String>, details: String) -> Result<HistoryEntry, String> {
    get_db().log_event(&event_type, player.as_deref(), &details).map_err(|e| e.to_string())
}
