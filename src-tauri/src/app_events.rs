use crate::services;
use crate::services::{ollama_kill, ollama_lifecycle};
use tauri::{Manager, RunEvent, WindowEvent};

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::ExitRequested { .. }
        | RunEvent::Exit
        | RunEvent::WindowEvent {
            event: WindowEvent::CloseRequested { .. },
            ..
        } => {
            if let Some(pty) = app_handle.try_state::<services::terminal::PtyManager>() {
                pty.kill_all();
            }
            ollama_kill::release_vram_blocking();
            ollama_lifecycle::stop_sidecar(app_handle);
        }
        _ => {}
    }
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
