use crate::services;
use crate::services::gateway::GatewayService;
use crate::services::{ollama_kill, ollama_lifecycle};
use tauri::{Manager, RunEvent, WindowEvent};

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { .. },
            ..
        } => {
            if label != "main" {
                return;
            }

            #[cfg(not(target_os = "macos"))]
            {
                if should_hide_instead_of_quit(app_handle) {
                    if let Some(win) = app_handle.get_webview_window("main") {
                        let _ = win.hide();
                    }
                } else {
                    cleanup(app_handle);
                }
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

#[cfg(not(target_os = "macos"))]
fn should_hide_instead_of_quit(app_handle: &tauri::AppHandle) -> bool {
    let config = services::config::read_config().unwrap_or_default();
    let gateway_active = config.gateway.enabled && config.gateway.run_when_window_closed;
    let tray_visible = config.advanced.show_tray;
    gateway_active && tray_visible
}

fn cleanup(app_handle: &tauri::AppHandle) {
    if let Some(gw) = app_handle.try_state::<GatewayService>() {
        let gw = gw.inner();
        tauri::async_runtime::block_on(async { gw.stop().await });
    }
    services::mcp_bridge::process_manager::shutdown_all();
    if let Some(pty) = app_handle.try_state::<services::terminal::PtyManager>() {
        pty.kill_all();
    }
    if let Some(chronos) = app_handle.try_state::<services::forecast::sidecar::ChronosSidecar>() {
        tauri::async_runtime::block_on(async {
            services::forecast::sidecar::stop(chronos.inner()).await;
        });
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
