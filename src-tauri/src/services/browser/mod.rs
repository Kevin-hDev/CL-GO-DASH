mod browser_api_types;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod browser_events;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod browser_slot;
mod browser_surface_api;
mod browser_view_key;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_app;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_blocked_feature;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_client;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_cookie_gate;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_cookie_gate_cleanup;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_display_handler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_download_handler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_engine;
#[cfg(target_os = "macos")]
mod cef_library;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_life_span_handler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_load_handler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_permission_handler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_request_handler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_state_bridge;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_surface;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_surface_view;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cef_text;
mod cookie_store_probe;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod ffi_guard;
mod lifecycle;
mod live_session_registry;
mod local_site_candidates;
mod local_site_policy;
mod local_site_probe;
mod local_site_scan_state;
mod local_site_scan_throttle;
mod local_site_scanner;
mod local_site_types;
#[cfg(target_os = "macos")]
mod native_application;
mod native_paths;
#[cfg(target_os = "macos")]
mod native_pump;
#[cfg(target_os = "macos")]
mod native_pump_wake;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native_surface;
mod navigation_target;
#[cfg(any(test, target_os = "macos"))]
mod process_role;
#[cfg(target_os = "macos")]
mod pump_gate;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod pump_scheduler;
mod runtime_handle;
mod runtime_revision;
mod session_model;
mod session_model_runtime;
mod session_persistence;
mod session_service;
mod session_store;
mod session_types;
mod session_validation;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod settings;
mod surface_bounds;
mod tab_id;
mod url_policy;
mod view_recency;
mod view_state;
#[cfg(target_os = "windows")]
pub(crate) mod windows_sandbox;

#[cfg(test)]
mod build_policy_tests;
#[cfg(all(test, target_os = "macos"))]
mod bundle_layout_tests;
#[cfg(test)]
mod cef_cookie_gate_policy_tests;
#[cfg(test)]
mod cookie_store_probe_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod ffi_guard_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod live_session_registry_tests;
#[cfg(test)]
mod local_site_candidates_tests;
#[cfg(test)]
mod local_site_policy_tests;
#[cfg(test)]
mod local_site_probe_tests;
#[cfg(test)]
mod local_site_scan_state_tests;
#[cfg(test)]
mod local_site_scan_throttle_tests;
#[cfg(test)]
mod native_paths_tests;
#[cfg(all(test, target_os = "macos"))]
mod native_pump_policy_tests;
#[cfg(test)]
mod navigation_target_tests;
#[cfg(test)]
mod process_role_tests;
#[cfg(all(test, target_os = "macos"))]
mod pump_gate_tests;
#[cfg(test)]
mod runtime_handle_tests;
#[cfg(test)]
mod runtime_revision_tests;
#[cfg(test)]
mod session_model_tests;
#[cfg(test)]
mod session_store_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod settings_tests;
#[cfg(test)]
mod surface_bounds_tests;
#[cfg(test)]
mod url_policy_tests;
#[cfg(test)]
mod view_recency_tests;
#[cfg(test)]
mod view_state_tests;
#[cfg(test)]
mod windows_bundle_layout_tests;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

pub use browser_api_types::{BrowserCommandError, BrowserNavigationAction, BrowserSurfaceRequest};
pub use browser_surface_api::{
    apply_surface, close_native_view, navigate_native_view, run_navigation_action,
};
pub use local_site_scanner::LocalSiteScanner;
pub use local_site_types::{LocalSiteScanResult, LOCAL_SITES_CHANGED_EVENT};
pub use runtime_handle::{BrowserCapability, BrowserRuntimeHandle};
pub use session_model::{BrowserSessionState, BrowserTabCreation};
pub use session_service::BrowserSessionService;

static NATIVE_APPLICATION_READY: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub(crate) fn prepare_native_application() -> bool {
    let ready = native_application::prepare().is_ok();
    NATIVE_APPLICATION_READY.store(ready, Ordering::Release);
    ready
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn prepare_native_application() -> bool {
    NATIVE_APPLICATION_READY.store(true, Ordering::Release);
    true
}

pub(crate) fn setup_on_run_event(app: &tauri::AppHandle, event: &tauri::RunEvent) {
    if !matches!(event, tauri::RunEvent::Ready) {
        return;
    }
    let runtime = app.state::<BrowserRuntimeHandle>().inner().clone();
    if !NATIVE_APPLICATION_READY.load(Ordering::Acquire) || !runtime.mark_application_prepared() {
        return;
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    cef_engine::initialize(app.clone(), runtime);
}

pub fn capability(app: &tauri::AppHandle) -> BrowserCapability {
    capability_for_runtime(app.state::<BrowserRuntimeHandle>().inner())
}

pub(super) fn capability_for_runtime(runtime: &BrowserRuntimeHandle) -> BrowserCapability {
    if cfg!(target_os = "linux") {
        BrowserCapability::Hidden
    } else {
        let runtime_capability = runtime.capability();
        if matches!(runtime_capability, BrowserCapability::Ready { .. })
            && session_store::session_key().is_err()
        {
            BrowserCapability::Unavailable
        } else {
            runtime_capability
        }
    }
}

pub(crate) fn shutdown(app: &tauri::AppHandle) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    cef_engine::shutdown(app.state::<BrowserRuntimeHandle>().inner());
}
