mod commands;
mod models;
mod services;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            services::file_watcher::start(app.handle());
            services::session_tail::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::list_wakeups,
            commands::create_wakeup,
            commands::update_wakeup,
            commands::delete_wakeup,
            commands::get_heartbeat_config,
            commands::set_heartbeat_active,
            commands::set_stop_at,
            commands::run_wakeup,
            commands::get_session_status,
            commands::get_warnings,
            commands::list_sessions,
            commands::get_session_detail,
            commands::rename_session,
            commands::delete_session_file,
            commands::list_personality_files,
            commands::read_personality_file,
            commands::open_in_editor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
