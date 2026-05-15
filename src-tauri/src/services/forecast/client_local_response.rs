use crate::services::forecast::input_data::ParsedInput;
use crate::services::forecast::registry::{find_runtime, ForecastEngineKind};
use crate::services::forecast::target_domain;
use crate::services::forecast::types::{ForecastRequest, ForecastResult, Prediction, Quantiles};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

type ParsedPrediction = (Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>);

pub fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let model_name = request
        .model
        .as_deref()
        .ok_or("Aucun modèle Forecast sélectionné")?;
    let runtime = find_runtime(model_name).ok_or("Moteur indisponible")?;
    let (mut predictions, mut q10, mut q50, mut q90) = match runtime.engine_kind {
        ForecastEngineKind::LocalChronos2
        | ForecastEngineKind::LocalTimesFm
        | ForecastEngineKind::LocalToto
        | ForecastEngineKind::LocalMoirai
        | ForecastEngineKind::LocalFlowState
        | ForecastEngineKind::LocalTabPfnTs
        | ForecastEngineKind::LocalTiRex
        | ForecastEngineKind::LocalKairos
        | ForecastEngineKind::LocalSundial => {
            parse_structured_or_simple_response(body, request, input)?
        }
        _ => parse_simple_response(body, input)?,
    };
    target_domain::apply_non_negative_floor(
        request,
        input,
        &mut predictions,
        &mut q10,
        &mut q50,
        &mut q90,
    );

    Ok(ForecastResult {
        id: Uuid::new_v4().to_string(),
        name: format!("Forecast {}", &request.target_column),
        target_column: request.target_column.clone(),
        created_at: Utc::now().to_rfc3339(),
        session_id: session_id.map(|s| s.to_string()),
        model: model_name.to_string(),
        provider: runtime.family_id.to_string(),
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

fn parse_simple_response(body: &Value, input: &ParsedInput) -> Result<ParsedPrediction, String> {
    let median = body["median"]
        .as_array()
        .ok_or("Réponse Forecast: champ median manquant")?;

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

fn parse_structured_or_simple_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<ParsedPrediction, String> {
    if body["predictions"].is_array() {
        return parse_structured_predictions(body, input);
    }
    if request.series_column.is_some() {
        return Err("Réponse Forecast multi-séries invalide".into());
    }
    parse_simple_response(body, input)
}

fn parse_structured_predictions(
    body: &Value,
    input: &ParsedInput,
) -> Result<ParsedPrediction, String> {
    let items = body["predictions"]
        .as_array()
        .ok_or("Réponse Forecast: champ predictions manquant")?;

    let mut predictions = Vec::with_capacity(items.len());
    let mut q10 = Vec::new();
    let mut q50 = Vec::with_capacity(items.len());
    let mut q90 = Vec::new();
    let mut has_all_q10 = true;
    let mut has_all_q90 = true;

    for item in items {
        let raw_date = item["date"]
            .as_str()
            .ok_or("Réponse Forecast: date manquante")?;
        let value = item["value"]
            .as_f64()
            .ok_or("Réponse Forecast: valeur manquante")?;
        predictions.push(Prediction {
            date: output_date(raw_date, predictions.len(), input),
            value,
            series_id: item["series_id"].as_str().map(|value| value.to_string()),
        });
        match item["q10"].as_f64() {
            Some(value) if has_all_q10 => q10.push(value),
            _ => has_all_q10 = false,
        }
        q50.push(item["q50"].as_f64().unwrap_or(value));
        match item["q90"].as_f64() {
            Some(value) if has_all_q90 => q90.push(value),
            _ => has_all_q90 = false,
        }
    }

    if !has_all_q10 || q10.len() != predictions.len() {
        q10.clear();
    }
    if !has_all_q90 || q90.len() != predictions.len() {
        q90.clear();
    }

    Ok((predictions, q10, q50, q90))
}

fn output_date(raw_date: &str, index: usize, input: &ParsedInput) -> String {
    if raw_date.trim().to_ascii_uppercase().starts_with("T+") {
        return input
            .future_dates
            .get(index)
            .cloned()
            .unwrap_or_else(|| raw_date.to_string());
    }
    raw_date.to_string()
}

fn extract_quantile_array(body: &Value, key: &str) -> Vec<f64> {
    body[key]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default()
}
