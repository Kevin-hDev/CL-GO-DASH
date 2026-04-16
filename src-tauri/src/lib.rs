mod commands;
mod models;
mod services;

use services::agent_local::ollama_client::OllamaClient;
use services::ollama_lifecycle::{self, OllamaSidecar};
use services::scheduler::Scheduler;
use std::collections::HashMap;
use tauri::{Manager, RunEvent, WindowEvent};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct ActiveStreams(pub Mutex<HashMap<String, CancellationToken>>);

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(OllamaClient::new())
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .manage(OllamaSidecar::new())
        .setup(|app| {
            if let Err(e) = migrate_legacy_storage() {
                eprintln!("[storage migration] {}", e);
            }
            // Migration one-shot : ancien keyring user "brave_api_key" → "brave"
            services::agent_local::tool_web_search::migrate_legacy_brave_key();
            if let Err(e) = ollama_lifecycle::start_sidecar(app.handle()) {
                eprintln!("[ollama] sidecar start failed: {}", e);
            }
            services::file_watcher::start(app.handle());
            let scheduler = Scheduler::spawn(app.handle().clone());
            app.manage(scheduler);
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
            commands::set_wakeup_active,
            commands::set_global_paused,
            commands::get_heartbeat_config,
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
            commands::translate_description,
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
            // Search providers
            commands::list_search_providers_catalog,
            commands::test_search_connection,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        RunEvent::ExitRequested { .. } => {
            ollama_lifecycle::stop_sidecar(app_handle);
        }
        RunEvent::WindowEvent { event: WindowEvent::CloseRequested { .. }, .. } => {
            ollama_lifecycle::stop_sidecar(app_handle);
        }
        _ => {}
    });
}

/// One-shot migration: copie depuis l'ancien dossier ~/.local/share/cl-go/
/// (utilisé par CL-GO) vers ~/.local/share/cl-go-dash/ au premier démarrage
/// de cette nouvelle version. L'ancien dossier est laissé intact (CL-GO continue
/// d'y écrire). N'écrase rien si le nouveau dossier existe déjà.
fn migrate_legacy_storage() -> Result<(), String> {
    use std::fs;

    let home = dirs::home_dir().ok_or("cannot resolve home")?;
    let new = home.join(".local/share/cl-go-dash");

    fs::create_dir_all(new.join("logs"))
        .map_err(|e| format!("create logs dir: {}", e))?;

    // Migration 1 : ~/.local/share/cl-go/ (legacy CL-GO)
    let cl_go_legacy = home.join(".local/share/cl-go");
    let legacy_marker = new.join(".migrated-from-cl-go");
    if !legacy_marker.exists() && cl_go_legacy.exists() {
        copy_items(&cl_go_legacy, &new);
        let _ = fs::write(&legacy_marker, b"ok");
    }

    // Migration 2 : ~/Library/Application Support/cl-go-dash/ (bug Phase 2 macOS)
    let app_support_wrong = dirs::data_local_dir().and_then(|d| {
        let p = d.join("cl-go-dash");
        if p != new { Some(p) } else { None }
    });
    let appsupport_marker = new.join(".migrated-from-appsupport");
    if let Some(wrong) = app_support_wrong {
        if !appsupport_marker.exists() && wrong.exists() {
            copy_items(&wrong, &new);
            let _ = fs::write(&appsupport_marker, b"ok");
        }
    }

    Ok(())
}

fn copy_items(src: &std::path::Path, dst: &std::path::Path) {
    let items: &[&str] = &[
        "agent-sessions",
        "agent-settings.json",
        "agent-tabs.json",
        "config.json",
        "memory",
        "inbox",
        "translations",
        "logs",
    ];
    for item in items {
        let s = src.join(item);
        let d = dst.join(item);
        if !s.exists() || d.exists() {
            continue;
        }
        if let Err(e) = copy_recursive(&s, &d) {
            eprintln!("[storage migration] {} → {}: {}", s.display(), d.display(), e);
        }
    }
}

fn copy_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use std::fs;
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
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
