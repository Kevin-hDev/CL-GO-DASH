use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::types::{ForecastRequest, ForecastResult, Prediction, Quantiles};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

pub async fn predict(
    base_url: &str,
    request: &ForecastRequest,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let input = parse_request_input(request)?;
    let payload = build_payload(&input.values, request);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/predict"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("Erreur sidecar Chronos: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        // Ne pas exposer le body brut — log interne uniquement
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[chronos] erreur {status}: {body}");
        return Err("Erreur du service de prédiction".to_string());
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("Parsing réponse Chronos: {e}"))?;

    parse_response(&body, request, &input, session_id)
}

pub async fn health_check(base_url: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/health"))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Health check échoué: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Sidecar non prêt ({})", resp.status()))
    }
}

fn build_payload(values: &[f64], request: &ForecastRequest) -> Value {
    let model = request.model.as_deref().unwrap_or("chronos-bolt-small");

    serde_json::json!({
        "values": values.to_vec(),
        "horizon": request.horizon,
        "model": model,
        "quantiles": [0.1, 0.5, 0.9],
    })
}

fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let median = body["median"]
        .as_array()
        .ok_or("Réponse Chronos: champ median manquant")?;

    let predictions: Vec<Prediction> = median
        .iter()
        .enumerate()
        .map(|(i, v)| Prediction {
            date: input
                .future_dates
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("T+{}", i + 1)),
            value: v.as_f64().unwrap_or(0.0),
        })
        .collect();

    let q10 = extract_quantile_array(body, "q10");
    let q50: Vec<f64> = predictions.iter().map(|p| p.value).collect();
    let q90 = extract_quantile_array(body, "q90");

    let model_name = request.model.as_deref().unwrap_or("chronos-bolt-small");

    Ok(ForecastResult {
        id: Uuid::new_v4().to_string(),
        name: format!("Forecast {}", &request.target_column),
        target_column: request.target_column.clone(),
        created_at: Utc::now().to_rfc3339(),
        session_id: session_id.map(|s| s.to_string()),
        model: model_name.to_string(),
        provider: "chronos".to_string(),
        horizon: request.horizon,
        frequency: request.frequency.clone(),
        input_summary: input.summary.clone(),
        input_data: input.snapshot.clone(),
        predictions,
        quantiles: Quantiles { q10, q50, q90 },
        covariates_used: request.covariate_columns.clone(),
        metrics: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
    })
}

fn extract_quantile_array(body: &Value, key: &str) -> Vec<f64> {
    body[key]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default()
}
