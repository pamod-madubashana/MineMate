use crate::memory::db::Database;
use std::sync::OnceLock;

static DB: OnceLock<Database> = OnceLock::new();

fn get_db() -> &'static Database {
    DB.get_or_init(|| Database::new("database/minemate.db").expect("Failed to initialize database"))
}

#[tauri::command]
pub async fn list_players() -> Result<Vec<String>, String> {
    get_db().list_players().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_locations() -> Result<Vec<String>, String> {
    get_db().list_locations().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_blueprints() -> Result<Vec<String>, String> {
    get_db().list_blueprints().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(limit: u32) -> Result<Vec<String>, String> {
    get_db().get_history(limit).map_err(|e| e.to_string())
}
