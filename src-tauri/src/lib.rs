mod commands;
mod models;
mod services;

use services::agent_local::ollama_client::OllamaClient;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct ActiveStreams(pub Mutex<HashMap<String, CancellationToken>>);

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(OllamaClient::new())
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .setup(|app| {
            services::file_watcher::start(app.handle());
            services::session_tail::start(app.handle());
            start_ollama_polling(app.handle().clone());
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
            // Agent Local — Ollama
            commands::list_ollama_models,
            commands::show_ollama_model,
            commands::is_ollama_running,
            commands::search_ollama_models,
            commands::get_registry_model_details,
            commands::list_registry_tags,
            commands::pull_ollama_model,
            commands::delete_ollama_model,
            commands::get_modelfile,
            commands::update_modelfile,
            commands::update_system_prompt,
            commands::update_parameters,
            // Agent Local — Chat + Sessions
            commands::chat_stream,
            commands::cancel_agent_request,
            commands::list_agent_sessions,
            commands::get_agent_session,
            commands::save_agent_session,
            commands::add_messages_to_session,
            commands::create_agent_session,
            commands::rename_agent_session,
            commands::delete_agent_session,
            commands::export_agent_session_markdown,
            commands::truncate_session_at,
            commands::truncate_and_replace_at,
            commands::get_tab_state,
            commands::save_tab_state,
            // Agent Local — Tools
            commands::list_skills,
            commands::load_skill,
            commands::set_brave_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_ollama_polling(handle: tauri::AppHandle) {
    use std::time::Duration;
    use tauri::Emitter;

    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_default();
        let mut last_running = false;

        loop {
            let running = client
                .get("http://localhost:11434/api/tags")
                .send()
                .await
                .is_ok();

            if running != last_running {
                let _ = handle.emit("ollama-status", running);
                last_running = running;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
