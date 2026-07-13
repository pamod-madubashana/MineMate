mod ai;
mod blueprint;
mod bot;
mod builder;
mod commands;
mod config;
mod executor;
mod memory;
mod task_engine;

pub fn run() {
    let _ = tracing_subscriber::fmt().try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::bot::start_bot,
            commands::bot::stop_bot,
            commands::bot::get_bot_status,
            commands::bot::send_chat,
            commands::bot::get_connection_status,
            commands::bot::get_audit_logs,
            commands::bot::follow_player,
            commands::bot::stop_following,
            commands::bot::set_guard_mode,
            commands::bot::get_guard_status,
            commands::bot::set_master_player,
            commands::config::get_config,
            commands::config::save_config,
            commands::memory::list_players,
            commands::memory::save_player,
            commands::memory::list_locations,
            commands::memory::save_location,
            commands::memory::list_blueprints,
            commands::memory::save_blueprint,
            commands::memory::get_history,
            commands::memory::log_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MineMate");
}
