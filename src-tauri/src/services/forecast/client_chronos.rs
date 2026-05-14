use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::registry::{find_runtime, ForecastEngineKind};
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
    let payload = build_payload(&input, request)?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/predict"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|_| "Erreur du service de prédiction".to_string())?;

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
        .map_err(|_| "Réponse du service de prédiction invalide".to_string())?;

    parse_response(&body, request, &input, session_id)
}

fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    let model = request.model.as_deref().unwrap_or("chronos-bolt-small");
    let runtime = find_runtime(model).ok_or("Moteur indisponible")?;

    Ok(match runtime.engine_kind {
        ForecastEngineKind::LocalChronosBolt => serde_json::json!({
            "values": input.values.to_vec(),
            "horizon": request.horizon,
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
            "model": model,
            "quantiles": [0.1, 0.5, 0.9],
        }),
        ForecastEngineKind::CloudApi => {
            return Err("Moteur local invalide".into());
        }
    })
}

fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let model_name = request.model.as_deref().unwrap_or("chronos-bolt-small");
    let runtime = find_runtime(model_name).ok_or("Moteur indisponible")?;
    let (predictions, q10, q50, q90) = match runtime.engine_kind {
        ForecastEngineKind::LocalChronos2 => parse_chronos2_response(body, request, input)?,
        _ => parse_simple_response(body, input)?,
    };

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

fn parse_simple_response(
    body: &Value,
    input: &ParsedInput,
) -> Result<(Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
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
            series_id: None,
        })
        .collect();

    let q10 = extract_quantile_array(body, "q10");
    let q50: Vec<f64> = predictions.iter().map(|p| p.value).collect();
    let q90 = extract_quantile_array(body, "q90");
    Ok((predictions, q10, q50, q90))
}

fn parse_chronos2_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<(Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
    if body["predictions"].is_array() {
        return parse_chronos2_structured_predictions(body);
    }
    if request.series_column.is_some() {
        return Err("Réponse Chronos multi-séries invalide".into());
    }
    parse_simple_response(body, input)
}

fn parse_chronos2_structured_predictions(
    body: &Value,
) -> Result<(Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
    let items = body["predictions"]
        .as_array()
        .ok_or("Réponse Chronos: champ predictions manquant")?;

    let mut predictions = Vec::with_capacity(items.len());
    let mut q10 = Vec::with_capacity(items.len());
    let mut q50 = Vec::with_capacity(items.len());
    let mut q90 = Vec::with_capacity(items.len());

    for item in items {
        let date = item["date"]
            .as_str()
            .ok_or("Réponse Chronos: date manquante")?;
        let value = item["value"]
            .as_f64()
            .ok_or("Réponse Chronos: valeur manquante")?;
        predictions.push(Prediction {
            date: date.to_string(),
            value,
            series_id: item["series_id"].as_str().map(|value| value.to_string()),
        });
        q10.push(item["q10"].as_f64().unwrap_or(value));
        q50.push(item["q50"].as_f64().unwrap_or(value));
        q90.push(item["q90"].as_f64().unwrap_or(value));
    }

    Ok((predictions, q10, q50, q90))
}

fn extract_quantile_array(body: &Value, key: &str) -> Vec<f64> {
    body[key]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default()
}
