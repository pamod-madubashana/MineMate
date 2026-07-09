use tauri::Manager;

mod ai;
mod commands;
mod config;
mod executor;
mod memory;
mod bot;

pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = bot::start_bot_loop(app_handle).await {
                    tracing::error!("Bot loop error: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bot::start_bot,
            commands::bot::stop_bot,
            commands::bot::get_bot_status,
            commands::bot::send_chat,
            commands::config::get_config,
            commands::config::save_config,
            commands::memory::list_players,
            commands::memory::list_locations,
            commands::memory::list_blueprints,
            commands::memory::get_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MineMate AI");
}
