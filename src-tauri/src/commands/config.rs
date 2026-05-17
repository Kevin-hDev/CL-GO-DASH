use crate::models::{AdvancedSettings, ClgoConfig};
use crate::services::config as config_service;

#[tauri::command]
pub fn get_config() -> Result<ClgoConfig, String> {
    config_service::read_config()
}

#[tauri::command]
pub fn save_config(mut config: ClgoConfig) -> Result<(), String> {
    let current = config_service::read_config()?;
    config.advanced = protect_advanced_settings(config.advanced, &current);
    config_service::write_config(&config)
}

#[tauri::command]
pub fn get_advanced_settings() -> Result<AdvancedSettings, String> {
    let config = config_service::read_config()?;
    Ok(config.advanced)
}

#[tauri::command]
pub fn set_advanced_settings(
    app: tauri::AppHandle,
    settings: AdvancedSettings,
) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    let manager = app.autolaunch();
    let current = manager.is_enabled().unwrap_or(false);
    if settings.autostart && !current {
        let _ = manager.enable();
    } else if !settings.autostart && current {
        let _ = manager.disable();
    }

    let mut config = config_service::read_config()?;
    config.advanced = protect_advanced_settings(settings, &config);
    config_service::write_config(&config)
}

fn protect_advanced_settings(
    mut settings: AdvancedSettings,
    current: &ClgoConfig,
) -> AdvancedSettings {
    settings.allowed_paths = current.advanced.allowed_paths.clone();
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protect_advanced_settings_keeps_existing_allowed_paths() {
        let mut current = ClgoConfig::default();
        current.advanced.allowed_paths = vec!["/trusted".to_string()];

        let incoming = AdvancedSettings {
            allowed_paths: vec!["/attacker".to_string()],
            ..Default::default()
        };

        let protected = protect_advanced_settings(incoming, &current);
        assert_eq!(protected.allowed_paths, vec!["/trusted"]);
    }
}

const PATCH_BLOCKED_KEYS: &[&str] = &["allowed_paths"];

#[tauri::command]
pub fn patch_advanced_settings(
    app: tauri::AppHandle,
    patch: serde_json::Value,
) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    let mut config = config_service::read_config()?;
    let mut current = serde_json::to_value(&config.advanced).map_err(|e| {
        eprintln!("[config] serialize: {e}");
        "Erreur de configuration".to_string()
    })?;

    if let (Some(base), Some(updates)) = (current.as_object_mut(), patch.as_object()) {
        for (k, v) in updates {
            if PATCH_BLOCKED_KEYS.contains(&k.as_str()) {
                continue;
            }
            base.insert(k.clone(), v.clone());
        }
    }

    let merged: AdvancedSettings = serde_json::from_value(current).map_err(|e| {
        eprintln!("[config] deserialize: {e}");
        "Erreur de configuration".to_string()
    })?;

    let manager = app.autolaunch();
    let enabled = manager.is_enabled().unwrap_or(false);
    if merged.autostart && !enabled {
        let _ = manager.enable();
    } else if !merged.autostart && enabled {
        let _ = manager.disable();
    }

    config.advanced = merged;
    config_service::write_config(&config)
}

#[tauri::command]
pub fn get_effective_context_length() -> u32 {
    crate::services::gpu_detect::compute_default_num_ctx()
}
