mod app_events;
mod commands;
mod models;
mod ollama_polling;
mod services;
mod storage_migration;
mod tray;

use services::agent_local::ollama_client::OllamaClient;
use services::ollama_lifecycle::{self, OllamaSidecar};
use services::scheduler::Scheduler;
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct ActiveStreams(pub Mutex<HashMap<String, (CancellationToken, u64)>>);
pub struct PullCancel(pub Mutex<Option<CancellationToken>>);

static STREAM_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .manage(OllamaClient::new())
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .manage(PullCancel(Mutex::new(None)))
        .manage(OllamaSidecar::new())
        .manage(services::terminal::PtyManager::new())
        .setup(|app| {
            if let Err(e) = storage_migration::run() {
                eprintln!("[storage migration] {}", e);
            }
            if let Err(e) = services::api_keys::init() {
                eprintln!("[vault] init failed: {}", e);
                #[cfg(target_os = "linux")]
                {
                    let handle = app.handle().clone();
                    let msg = e.to_string();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        let _ = handle.emit("vault-init-failed", msg);
                    });
                }
            }
            if ollama_lifecycle::ollama_binary_path().is_ok() {
                if let Err(e) = ollama_lifecycle::start_sidecar(app.handle()) {
                    eprintln!("[ollama] sidecar start failed: {}", e);
                }
            }

            let config = services::config::read_config().unwrap_or_default();

            // Autostart : synchronise l'état OS avec le setting
            app_events::sync_autostart(app.handle(), config.advanced.autostart);

            // Start hidden : masque la fenêtre si activé
            if config.advanced.start_hidden {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            // Linux/Windows : décorations standard (pas de titlebar overlay)
            #[cfg(not(target_os = "macos"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_decorations(true);
                }
            }

            // Tray icon
            if config.advanced.show_tray {
                let _ = tray::create_tray(app);
            }

            services::file_watcher::start(app.handle());
            let scheduler = Scheduler::spawn(app.handle().clone());
            app.manage(scheduler);
            ollama_polling::start(app.handle().clone());
            tauri::async_runtime::spawn(services::llm::model_registry::init());
            Ok(())
        })
        .on_window_event(|_app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(win) = _app.get_webview_window("main") {
                    let _ = win.hide();
                }
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_advanced_settings,
            commands::set_advanced_settings,
            commands::get_effective_context_length,
            commands::list_wakeups,
            commands::create_wakeup,
            commands::update_wakeup,
            commands::delete_wakeup,
            commands::set_wakeup_active,
            commands::set_global_paused,
            commands::get_heartbeat_config,
            commands::list_personality_files,
            commands::read_personality_file,
            commands::open_in_editor,
            commands::get_injection_state,
            commands::set_injection_state,
            // Agent Local — Ollama
            commands::list_ollama_models,
            commands::show_ollama_model,
            commands::is_ollama_running,
            commands::search_ollama_models,
            commands::get_registry_model_details,
            commands::list_registry_tags,
            commands::translate_description,
            commands::pull_ollama_model,
            commands::cancel_pull_ollama_model,
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
            commands::update_session_model,
            commands::delete_agent_session,
            commands::export_agent_session_markdown,
            commands::truncate_session_at,
            commands::truncate_and_replace_at,
            commands::get_tab_state,
            commands::save_tab_state,
            commands::get_agent_settings,
            commands::set_permission_mode,
            commands::respond_to_permission,
            // Agent Local — Tools
            commands::list_skills,
            commands::load_skill,
            commands::set_brave_api_key,
            // API Keys (multi-provider)
            commands::set_api_key,
            commands::delete_api_key,
            commands::has_api_key,
            commands::list_configured_providers,
            commands::test_api_key,
            // LLM providers
            commands::list_llm_providers_catalog,
            commands::list_llm_models,
            commands::test_llm_connection,
            commands::supports_tool_use,
            commands::get_provider_quota,
            // Search providers
            commands::list_search_providers_catalog,
            commands::test_search_connection,
            // LLM Registry
            commands::search_registry,
            commands::get_registry_model,
            commands::list_registry_families,
            commands::list_family_models,
            // Favorite models
            commands::list_favorite_models,
            commands::add_favorite_model,
            commands::remove_favorite_model,
            // Projects
            commands::list_projects,
            commands::add_project,
            commands::rename_project,
            commands::delete_project,
            commands::reorder_projects,
            commands::open_project_folder,
            // Agent MD
            commands::read_agent_md,
            commands::write_agent_md,
            // Terminal PTY
            commands::pty_spawn,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_kill,
            // Updates
            commands::check_app_update,
            commands::download_app_update,
            commands::check_ollama_updates,
            // Ollama setup
            commands::is_ollama_installed,
            commands::download_ollama,
            commands::start_ollama_sidecar,
            commands::restart_ollama_sidecar,
            commands::check_model_fits_vram,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| app_events::handle_run_event(app_handle, event));
}

