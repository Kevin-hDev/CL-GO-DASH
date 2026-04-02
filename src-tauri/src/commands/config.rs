use crate::models::ClgoConfig;
use crate::services::config as config_service;

#[tauri::command]
pub fn get_config() -> Result<ClgoConfig, String> {
    config_service::read_config()
}

#[tauri::command]
pub fn save_config(config: ClgoConfig) -> Result<(), String> {
    config_service::write_config(&config)
}
