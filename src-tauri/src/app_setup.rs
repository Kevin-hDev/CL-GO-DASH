use crate::{ollama_polling, services, storage_migration, tray};
use tauri::Manager;

pub fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = storage_migration::run() {
        eprintln!("[storage migration] {}", e);
    }
    if let Err(e) = services::api_keys::init() {
        eprintln!("[vault] init failed: {}", e);
        emit_vault_error(app, e.to_string());
    }
    if services::ollama_lifecycle::ollama_binary_path().is_ok() {
        if let Err(e) = services::ollama_lifecycle::start_sidecar(app.handle()) {
            eprintln!("[ollama] sidecar start failed: {}", e);
        }
    }

    let config = services::config::read_config().unwrap_or_default();
    crate::app_events::sync_autostart(app.handle(), config.advanced.autostart);

    if config.advanced.start_hidden {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.hide();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.set_decorations(false);
            let _ = win.set_shadow(false);
        }
    }

    if config.advanced.show_tray {
        let _ = tray::create_tray(app);
    }

    services::file_watcher::start(app.handle());
    let scheduler = services::scheduler::Scheduler::spawn(app.handle().clone());
    app.manage(scheduler);
    ollama_polling::start(app.handle().clone());
    tauri::async_runtime::spawn(services::llm::model_registry::init());
    Ok(())
}

fn emit_vault_error(_app: &mut tauri::App, msg: String) {
    #[cfg(target_os = "linux")]
    {
        use tauri::Emitter;
        let handle = _app.handle().clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = handle.emit("vault-init-failed", msg);
        });
    }

    #[cfg(not(target_os = "linux"))]
    let _ = msg;
}
