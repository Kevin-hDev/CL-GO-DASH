use crate::services::forecast::client_nixtla_retry;
use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::nixtla_multiseries;
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
    let payload = build_payload(&input, request)?;
    let endpoint = if request.series_column.is_some() {
        format!("{API_BASE}/v2/forecast")
    } else {
        format!("{API_BASE}/forecast")
    };

    let client = reqwest::Client::new();
    let resp =
        client_nixtla_retry::post_json_with_retry(&client, &endpoint, api_key, &payload).await?;

    if !resp.status().is_success() {
        let status = resp.status();
        eprintln!("[nixtla] erreur {status}");
        return Err("Erreur du service de prédiction".to_string());
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|_| "Réponse du service de prédiction invalide".to_string())?;

    parse_response(&body, request, &input, session_id)
}

fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    if request.series_column.is_some() {
        return nixtla_multiseries::build_payload(input, request);
    }
    let timestamps: Vec<&str> = input
        .snapshot
        .history
        .iter()
        .map(|point| point.date.as_str())
        .collect();
    let model = request.model.as_deref().unwrap_or("timegpt-2-standard");
    let model_config =
        crate::services::forecast::model_config::effective_values(model).unwrap_or_default();
    let level = model_config
        .get("level")
        .and_then(Value::as_u64)
        .unwrap_or((request.confidence_level * 100.0) as u64);

    let mut payload = serde_json::json!({
        "timestamp": timestamps,
        "value": input.values,
        "freq": &request.frequency,
        "fh": request.horizon,
        "model": model,
        "level": [level],
    });
    apply_timegpt_options(&mut payload, &model_config);
    Ok(payload)
}

fn apply_timegpt_options(payload: &mut Value, config: &serde_json::Map<String, Value>) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };
    for key in [
        "clean_ex_first",
        "finetune_steps",
        "finetune_loss",
        "finetune_depth",
        "feature_contributions",
    ] {
        if let Some(value) = config.get(key) {
            object.insert(key.to_string(), value.clone());
        }
    }
}

fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let (predictions, q10, q50, q90) = if request.series_column.is_some() {
        nixtla_multiseries::parse_response(body, request, input)?
    } else {
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
                series_id: None,
            })
            .collect();

        let q50: Vec<f64> = predictions.iter().map(|p| p.value).collect();
        (predictions, Vec::new(), q50, Vec::new())
    };

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
        quantiles: Quantiles { q10, q50, q90 },
        covariates_used: request.covariate_columns.clone(),
        metrics: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
    })
}
