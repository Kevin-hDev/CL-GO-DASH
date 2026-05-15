use crate::services::forecast::client_local_response;
use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::registry::{find_runtime, ForecastEngineKind};
use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use serde_json::Value;

pub async fn predict(
    base_url: &str,
    auth_token: &str,
    request: &ForecastRequest,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let input = parse_request_input(request)?;
    let payload = build_payload(&input, request)?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/predict"))
        .header("Content-Type", "application/json")
        .header("X-CLGO-Forecast-Token", auth_token)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|_| "Erreur du service de prédiction".to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        eprintln!("[chronos] erreur {status}");
        return Err("Erreur du service de prédiction".to_string());
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|_| "Réponse du service de prédiction invalide".to_string())?;

    client_local_response::parse_response(&body, request, &input, session_id)
}

fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    let model = request.model.as_deref().unwrap_or("chronos-bolt-small");
    let runtime = find_runtime(model).ok_or("Moteur indisponible")?;
    if !crate::services::forecast::registry::has_predict_adapter(runtime) {
        return Err("Moteur indisponible".into());
    }

    Ok(match runtime.engine_kind {
        ForecastEngineKind::LocalChronosBolt => serde_json::json!({
            "values": input.values.to_vec(),
            "horizon": request.horizon,
            "frequency": request.frequency,
            "model": model,
            "quantiles": [0.1, 0.5, 0.9],
        }),
        ForecastEngineKind::LocalChronos2 => serde_json::json!({
            "history_rows": input.history_rows,
            "future_rows": input.future_rows,
            "date_column": request.date_column,
            "target_column": request.target_column,
            "series_column": request.series_column,
            "covariate_columns": request.covariate_columns,
            "horizon": request.horizon,
            "frequency": request.frequency,
            "model": model,
            "quantiles": [0.1, 0.5, 0.9],
        }),
        ForecastEngineKind::LocalTimesFm
        | ForecastEngineKind::LocalToto
        | ForecastEngineKind::LocalMoirai
        | ForecastEngineKind::LocalFlowState
        | ForecastEngineKind::LocalTabPfnTs
        | ForecastEngineKind::LocalTiRex
        | ForecastEngineKind::LocalKairos
        | ForecastEngineKind::LocalSundial => serde_json::json!({
            "values": input.values.to_vec(),
            "history_rows": input.history_rows,
            "future_rows": input.future_rows,
            "date_column": request.date_column,
            "target_column": request.target_column,
            "series_column": request.series_column,
            "covariate_columns": request.covariate_columns,
            "horizon": request.horizon,
            "frequency": request.frequency,
            "model": model,
            "family": runtime.family_id,
            "quantiles": [0.1, 0.5, 0.9],
        }),
        ForecastEngineKind::CloudApi => {
            return Err("Moteur local invalide".into());
        }
    })
}
