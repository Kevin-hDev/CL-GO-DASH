use crate::models::{AdvancedSettings, ClgoConfig};
use crate::services::config as config_service;

#[tauri::command]
pub fn get_config() -> Result<ClgoConfig, String> {
    config_service::read_config()
}

#[tauri::command]
pub fn save_config(config: ClgoConfig) -> Result<(), String> {
    config_service::write_config(&config)
}

#[tauri::command]
pub fn get_advanced_settings() -> Result<AdvancedSettings, String> {
    let config = config_service::read_config()?;
    Ok(config.advanced)
}

#[tauri::command]
pub fn set_advanced_settings(settings: AdvancedSettings) -> Result<(), String> {
    let mut config = config_service::read_config()?;
    config.advanced = settings;
    config_service::write_config(&config)
}
