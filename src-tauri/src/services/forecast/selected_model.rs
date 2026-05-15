use super::{types::ForecastRequest, validation};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
struct SelectedForecastModel {
    model: String,
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("forecast-selected-model.json")
}

pub fn get() -> Option<String> {
    let content = std::fs::read_to_string(path()).ok()?;
    let selected: SelectedForecastModel = serde_json::from_str(&content).ok()?;
    validation::validate_runnable_model_id(&selected.model).ok()?;
    Some(selected.model)
}

pub fn require() -> Result<String, String> {
    get().ok_or_else(|| "Aucun modèle Forecast sélectionné".to_string())
}

pub fn set(model: &str) -> Result<(), String> {
    validation::validate_runnable_model_id(model)?;
    let target = path();
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| "Impossible d'enregistrer le modèle".to_string())?;
    }
    let selected = SelectedForecastModel {
        model: model.to_string(),
    };
    let content = serde_json::to_string_pretty(&selected)
        .map_err(|_| "Impossible d'enregistrer le modèle".to_string())?;
    let tmp = target.with_extension("tmp");
    std::fs::write(&tmp, content).map_err(|_| "Impossible d'enregistrer le modèle".to_string())?;
    std::fs::rename(&tmp, &target).map_err(|_| "Impossible d'enregistrer le modèle".to_string())
}

pub fn apply_required(request: &mut ForecastRequest) -> Result<String, String> {
    let model = require()?;
    request.model = Some(model.clone());
    Ok(model)
}
