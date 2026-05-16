use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

const MAX_MODELS: usize = 100;
const MAX_PARAMS_PER_MODEL: usize = 64;

pub type StoredConfigs = BTreeMap<String, Map<String, Value>>;

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("forecast-model-configs.json")
}

pub fn read_all() -> StoredConfigs {
    let content = match std::fs::read_to_string(path()) {
        Ok(content) => content,
        Err(_) => return BTreeMap::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn read_model(model_id: &str) -> Map<String, Value> {
    read_all().remove(model_id).unwrap_or_default()
}

pub fn write_model(model_id: &str, values: Map<String, Value>) -> Result<(), String> {
    let mut all = read_all();
    if values.is_empty() {
        all.remove(model_id);
    } else {
        if !all.contains_key(model_id) && all.len() >= MAX_MODELS {
            return Err("Configuration Forecast trop volumineuse".into());
        }
        if values.len() > MAX_PARAMS_PER_MODEL {
            return Err("Configuration Forecast invalide".into());
        }
        all.insert(model_id.to_string(), values);
    }
    write_all(&all)
}

fn write_all(all: &StoredConfigs) -> Result<(), String> {
    let target = path();
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| "Impossible d'enregistrer la configuration".to_string())?;
    }
    let body = serde_json::to_string_pretty(all)
        .map_err(|_| "Impossible d'enregistrer la configuration".to_string())?;
    let tmp = target.with_extension("tmp");
    std::fs::write(&tmp, body)
        .map_err(|_| "Impossible d'enregistrer la configuration".to_string())?;
    std::fs::rename(&tmp, &target)
        .map_err(|_| "Impossible d'enregistrer la configuration".to_string())
}
