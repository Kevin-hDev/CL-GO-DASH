use super::{
    client_chronos, client_nixtla, model_manager, sidecar,
    types::{ForecastRequest, ForecastResult},
    validation,
};
use serde_json::Value;

pub async fn rerun(
    analysis: &ForecastResult,
    rows: Vec<Value>,
    chronos: Option<&sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let request = build_forecast_request(analysis, rows)?;
    validation::validate_request(&request)?;
    let model_id = validation::model_id(&request)?;

    if model_id.starts_with("timegpt") {
        let key = crate::services::api_keys::get_key("nixtla")
            .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
        return client_nixtla::predict(&key, &request, None).await;
    }

    if !model_manager::is_installed(model_id) {
        return Err("Modèle non installé".into());
    }
    let chronos = chronos.ok_or("Service de prédiction indisponible")?;
    let endpoint = sidecar::start(chronos, model_id)
        .await
        .map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?;
    client_chronos::predict(
        &endpoint.base_url,
        endpoint.auth_token.as_str(),
        &request,
        None,
    )
    .await
}

fn build_forecast_request(
    analysis: &ForecastResult,
    rows: Vec<Value>,
) -> Result<ForecastRequest, String> {
    let date_column = analysis
        .input_data
        .date_column
        .clone()
        .or_else(|| infer_date_column(analysis))
        .ok_or("Colonne date introuvable")?;
    let data = serde_json::to_string(&rows).map_err(|_| "Erreur de sérialisation".to_string())?;

    Ok(ForecastRequest {
        data: Some(data),
        file_path: None,
        target_column: analysis.target_column.clone(),
        date_column,
        series_column: analysis.input_data.series_column.clone(),
        covariate_columns: analysis.covariates_used.clone(),
        horizon: analysis.horizon,
        frequency: analysis.frequency.clone(),
        model: Some(analysis.model.clone()),
        confidence_level: 0.9,
    })
}

fn infer_date_column(analysis: &ForecastResult) -> Option<String> {
    ["date", "timestamp", "time"]
        .iter()
        .find(|column| {
            analysis
                .input_data
                .columns
                .iter()
                .any(|item| item == **column)
        })
        .map(|column| (*column).to_string())
}
