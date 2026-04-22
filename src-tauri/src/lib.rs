mod commands;
mod models;
mod services;

use services::agent_local::ollama_client::OllamaClient;
use services::ollama_lifecycle::{self, OllamaSidecar};
use services::scheduler::Scheduler;
use std::collections::HashMap;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent, WindowEvent,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct ActiveStreams(pub Mutex<HashMap<String, CancellationToken>>);

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(OllamaClient::new())
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .manage(OllamaSidecar::new())
        .manage(services::terminal::PtyManager::new())
        .setup(|app| {
            if let Err(e) = migrate_legacy_storage() {
                eprintln!("[storage migration] {}", e);
            }
            if let Err(e) = services::api_keys::init() {
                eprintln!("[vault] init failed: {}", e);
            }
            if let Err(e) = ollama_lifecycle::start_sidecar(app.handle()) {
                eprintln!("[ollama] sidecar start failed: {}", e);
            }

            let config = services::config::read_config().unwrap_or_default();

            // Autostart : synchronise l'état OS avec le setting
            sync_autostart(app.handle(), config.advanced.autostart);

            // Start hidden : masque la fenêtre si activé
            if config.advanced.start_hidden {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            // Tray icon
            if config.advanced.show_tray {
                let _ = create_tray(app);
            }

            services::file_watcher::start(app.handle());
            let scheduler = Scheduler::spawn(app.handle().clone());
            app.manage(scheduler);
            start_ollama_polling(app.handle().clone());
            tauri::async_runtime::spawn(services::llm::model_registry::init());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_advanced_settings,
            commands::set_advanced_settings,
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
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        RunEvent::ExitRequested { .. }
        | RunEvent::Exit
        | RunEvent::WindowEvent { event: WindowEvent::CloseRequested { .. }, .. } => {
            if let Some(pty) = app_handle.try_state::<services::terminal::PtyManager>() {
                pty.kill_all();
            }
            ollama_lifecycle::stop_sidecar(app_handle);
        }
        _ => {}
    });
}

fn sync_autostart(handle: &tauri::AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let manager = handle.autolaunch();
    let current = manager.is_enabled().unwrap_or(false);
    if enabled && !current {
        let _ = manager.enable();
    } else if !enabled && current {
        let _ = manager.disable();
    }
}

fn tray_lang() -> &'static str {
    let lang_env = std::env::var("LANG").unwrap_or_default();
    if lang_env.to_lowercase().starts_with("fr") { "fr" } else { "en" }
}

fn create_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let (show_label, quit_label) = if tray_lang() == "fr" {
        ("Afficher", "Quitter")
    } else {
        ("Show", "Quit")
    };
    let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?)
        .menu(&menu)
        .tooltip("CL-GO")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { .. } = event {
                if let Some(win) = tray.app_handle().get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
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
                .get(format!("{}/api/tags", services::agent_local::OLLAMA_BASE_URL))
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
