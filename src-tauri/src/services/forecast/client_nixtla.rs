use crate::services::forecast::client_http;
use crate::services::forecast::client_nixtla_options;
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

    let client = client_http::internet_client()?;
    let resp =
        client_nixtla_retry::post_json_with_retry(&client, &endpoint, api_key, &payload).await?;

    if !resp.status().is_success() {
        let status = resp.status();
        eprintln!("[nixtla] erreur {status}");
        return Err("Erreur du service de prédiction".to_string());
    }

    let body: Value = client_http::read_json(resp).await?;

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
    let model_config = client_nixtla_options::effective_config(model);
    let level = client_nixtla_options::effective_level(request.confidence_level);

    let mut payload = serde_json::json!({
        "timestamp": timestamps,
        "value": input.values,
        "freq": &request.frequency,
        "fh": request.horizon,
        "model": model,
        "level": [level],
    });
    client_nixtla_options::apply(&mut payload, &model_config);
    Ok(payload)
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
            .map(|(index, value)| {
                Ok(Prediction {
                    date: input
                        .future_dates
                        .get(index)
                        .cloned()
                        .ok_or("Dates futures invalides")?,
                    value: value
                        .as_f64()
                        .filter(|number| number.is_finite())
                        .ok_or("Réponse Nixtla invalide")?,
                    series_id: None,
                })
            })
            .collect::<Result<_, String>>()?;

        let level = client_nixtla_options::effective_level(request.confidence_level);
        let q10 = client_nixtla_options::interval_array(body, &format!("lo-{level}"))?;
        let q50: Vec<f64> = predictions.iter().map(|p| p.value).collect();
        let q90 = client_nixtla_options::interval_array(body, &format!("hi-{level}"))?;
        (predictions, q10, q50, q90)
    };

    let model_name = request.model.as_deref().unwrap_or("timegpt-2-standard");

    let result = ForecastResult {
        id: Uuid::new_v4().to_string(),
        name: format!("Forecast {}", request.target_column),
        target_column: request.target_column.clone(),
        created_at: Utc::now().to_rfc3339(),
        session_id: session_id.map(|s| s.to_string()),
        model: model_name.to_string(),
        provider: "nixtla".to_string(),
        horizon: request.horizon,
        frequency: request.frequency.clone(),
        confidence_level: request.confidence_level,
        input_summary: input.summary.clone(),
        input_data: input.snapshot.clone(),
        data_profile: Some(input.data_profile.clone()),
        predictions,
        quantiles: Quantiles { q10, q50, q90 },
        covariates_used: request.covariate_columns.clone(),
        metrics: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
    };
    crate::services::forecast::result_validation::validate(&result, request, input)?;
    Ok(result)
}
