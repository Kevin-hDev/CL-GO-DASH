use crate::models::ClgoConfig;
use std::fs;
use std::path::PathBuf;

fn config_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home directory");
    home.join(".local/share/cl-go/config.json")
}

pub fn read_config() -> Result<ClgoConfig, String> {
    let path = config_path();
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read config: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid config JSON: {}", e))
}

pub fn write_config(config: &ClgoConfig) -> Result<(), String> {
    let path = config_path();
    let tmp_path = path.with_extension("json.tmp");

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Cannot serialize config: {}", e))?;

    // Atomic write: tmp + rename
    fs::write(&tmp_path, &content)
        .map_err(|e| format!("Cannot write tmp config: {}", e))?;
    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Cannot rename config: {}", e))?;

    Ok(())
}
