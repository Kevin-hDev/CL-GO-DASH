use crate::models::{AdvancedSettings, ClgoConfig, HeartbeatConfig, ScheduledWakeup};
use std::fs;
use std::path::PathBuf;

fn config_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home directory");
    home.join(".local/share/cl-go-dash/config.json")
}

/// Lecture tolérante du config :
/// - fichier absent → config par défaut (vide)
/// - JSON corrompu → config par défaut + log
/// - wakeups au format obsolète (CL-GO legacy) → ignorés un par un + log
pub fn read_config() -> Result<ClgoConfig, String> {
    let path = config_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Ok(ClgoConfig::default()),
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[config] JSON invalide ({}), reset à zéro", e);
            return Ok(ClgoConfig::default());
        }
    };

    let mut config = ClgoConfig::default();
    let Some(obj) = value.as_object() else { return Ok(config); };

    if let Some(hb) = obj.get("heartbeat") {
        config.heartbeat = serde_json::from_value::<HeartbeatConfig>(hb.clone())
            .unwrap_or_default();
    }

    if let Some(adv) = obj.get("advanced") {
        config.advanced = serde_json::from_value::<AdvancedSettings>(adv.clone())
            .unwrap_or_default();
    }

    if let Some(arr) = obj.get("scheduled_wakeups").and_then(|v| v.as_array()) {
        let mut dropped = 0u32;
        for item in arr {
            match serde_json::from_value::<ScheduledWakeup>(item.clone()) {
                Ok(w) => config.scheduled_wakeups.push(w),
                Err(_) => dropped += 1,
            }
        }
        if dropped > 0 {
            eprintln!("[config] {} wakeup(s) au format obsolète ignoré(s)", dropped);
        }
    }

    Ok(config)
}

pub fn write_config(config: &ClgoConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create config dir: {}", e))?;
    }
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
