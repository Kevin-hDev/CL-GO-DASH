use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::types::{ForecastRequest, ForecastResult, Prediction, Quantiles};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;
use zeroize::Zeroizing;

const API_BASE: &str = "https://api.nixtla.io";

pub async fn predict(
    api_key: &Zeroizing<String>,
    request: &ForecastRequest,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let input = parse_request_input(request)?;
    let payload = build_payload(&input, request);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{API_BASE}/forecast"))
        .header("Authorization", format!("Bearer {}", api_key.as_str()))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau Nixtla: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        // Ne pas exposer le body brut — log interne uniquement
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[nixtla] erreur {status}: {body}");
        return Err("Erreur du service de prédiction".to_string());
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("Parsing réponse Nixtla: {e}"))?;

    parse_response(&body, request, &input, session_id)
}

pub async fn test_connection(api_key: &Zeroizing<String>) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{API_BASE}/models"))
        .header("Authorization", format!("Bearer {}", api_key.as_str()))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Connexion Nixtla échouée: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Clé Nixtla invalide ({})", resp.status()))
    }
}

fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Value {
    let timestamps: Vec<&str> = input
        .snapshot
        .history
        .iter()
        .map(|point| point.date.as_str())
        .collect();
    let model = request.model.as_deref().unwrap_or("timegpt-2-standard");

    serde_json::json!({
        "timestamp": timestamps,
        "value": input.values,
        "freq": &request.frequency,
        "fh": request.horizon,
        "model": model,
        "level": [(request.confidence_level * 100.0) as u32],
    })
}

fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let forecasts = body["forecast"]
        .as_array()
        .or_else(|| body["value"].as_array())
        .ok_or("Réponse Nixtla: champ forecast manquant")?;

    let predictions: Vec<Prediction> = forecasts
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

    let q50: Vec<f64> = predictions.iter().map(|p| p.value).collect();

    let model_name = request.model.as_deref().unwrap_or("timegpt-2-standard");

    Ok(ForecastResult {
        id: Uuid::new_v4().to_string(),
        name: format!("Forecast {}", &request.target_column),
        target_column: request.target_column.clone(),
        created_at: Utc::now().to_rfc3339(),
        session_id: session_id.map(|s| s.to_string()),
        model: model_name.to_string(),
        provider: "nixtla".to_string(),
        horizon: request.horizon,
        frequency: request.frequency.clone(),
        input_summary: input.summary.clone(),
        input_data: input.snapshot.clone(),
        predictions,
        quantiles: Quantiles {
            q10: Vec::new(),
            q50,
            q90: Vec::new(),
        },
        covariates_used: request.covariate_columns.clone(),
        metrics: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
    })
}
