use crate::services;
use crate::services::{ollama_kill, ollama_lifecycle};
use tauri::{Manager, RunEvent, WindowEvent};

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            event: WindowEvent::CloseRequested { .. },
            ..
        } => {
            // macOS : on hide la fenêtre dans on_window_event, pas de cleanup ici
            #[cfg(not(target_os = "macos"))]
            {
                cleanup(app_handle);
            }
        }
        RunEvent::ExitRequested { .. } | RunEvent::Exit => {
            cleanup(app_handle);
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            if let Some(win) = app_handle.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
        _ => {}
    }
}

fn cleanup(app_handle: &tauri::AppHandle) {
    if let Some(pty) = app_handle.try_state::<services::terminal::PtyManager>() {
        pty.kill_all();
    }
    ollama_kill::release_vram_blocking();
    ollama_lifecycle::stop_sidecar(app_handle);
}

pub fn sync_autostart(handle: &tauri::AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let manager = handle.autolaunch();
    let current = manager.is_enabled().unwrap_or(false);
    if enabled && !current {
        let _ = manager.enable();
    } else if !enabled && current {
        let _ = manager.disable();
    }
}
