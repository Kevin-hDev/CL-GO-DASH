use super::{catalog, registry, types::ForecastRequest};

const MAX_DATA_BYTES: usize = 5 * 1024 * 1024;
const MAX_COLUMN_LEN: usize = 80;
const MAX_COVARIATES: usize = 64;
const MAX_MODEL_ID_LEN: usize = 80;
const ALLOWED_FREQUENCIES: &[&str] = &[
    "10S", "15S", "30S", "S", "T", "min", "H", "D", "B", "W", "M", "Q", "Y",
];

pub fn model_id(request: &ForecastRequest) -> Result<&str, String> {
    let id = request.model.as_deref().unwrap_or("chronos-bolt-small");
    validate_model_id(id)?;
    Ok(id)
}

pub fn validate_model_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_MODEL_ID_LEN {
        return Err("Modèle invalide".into());
    }
    if catalog::find_model(id).is_none() {
        return Err("Modèle inconnu".into());
    }
    if registry::find_runtime(id).is_none() {
        return Err("Moteur indisponible".into());
    }
    Ok(())
}

pub fn validate_request(request: &ForecastRequest) -> Result<(), String> {
    let model_id = model_id(request)?;
    let spec = catalog::find_model(model_id).ok_or("Modèle inconnu")?;
    let runtime = registry::find_runtime(model_id).ok_or("Moteur indisponible")?;

    validate_column(&request.target_column)?;
    validate_column(&request.date_column)?;
    if request.target_column == request.date_column {
        return Err("Colonnes invalides".into());
    }
    if let Some(series_column) = request.series_column.as_ref() {
        validate_column(series_column)?;
        if !runtime.capabilities.multivariate {
            return Err("Multi-séries non supporté par ce moteur".into());
        }
        if series_column == &request.target_column || series_column == &request.date_column {
            return Err("Colonne série invalide".into());
        }
    }
    if request.covariate_columns.len() > MAX_COVARIATES {
        return Err("Trop de covariables".into());
    }
    if !request.covariate_columns.is_empty() && !runtime.capabilities.past_covariates {
        return Err("Variables de contexte non supportées par ce moteur".into());
    }
    let mut unique_covariates = std::collections::BTreeSet::new();
    for column in &request.covariate_columns {
        validate_column(column)?;
        if column == &request.target_column
            || column == &request.date_column
            || request.series_column.as_ref() == Some(column)
        {
            return Err("Covariables invalides".into());
        }
        if !unique_covariates.insert(column) {
            return Err("Covariables invalides".into());
        }
    }
    if request.horizon == 0 || request.horizon > spec.horizon_max {
        return Err("Horizon invalide".into());
    }
    if !(0.5..=0.99).contains(&request.confidence_level) {
        return Err("Niveau de confiance invalide".into());
    }
    if !ALLOWED_FREQUENCIES.contains(&request.frequency.as_str()) {
        return Err("Fréquence invalide".into());
    }
    match (&request.data, &request.file_path) {
        (None, None) => Err("Données manquantes".into()),
        (Some(data), _) if data.len() > MAX_DATA_BYTES => Err("Données trop volumineuses".into()),
        _ => Ok(()),
    }
}

fn validate_column(column: &str) -> Result<(), String> {
    if column.is_empty() || column.len() > MAX_COLUMN_LEN {
        return Err("Colonne invalide".into());
    }
    if column
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\' | '\0'))
    {
        return Err("Colonne invalide".into());
    }
    Ok(())
}
