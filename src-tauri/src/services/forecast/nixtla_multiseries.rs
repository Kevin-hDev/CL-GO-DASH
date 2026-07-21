use super::client_nixtla_options;
use super::input_data::ParsedInput;
use super::types::{ForecastRequest, Prediction};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
type ParsedNixtla = (Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>);
pub fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    let series_column = request
        .series_column
        .as_deref()
        .ok_or("Colonne série manquante")?;
    let grouped_history = group_rows(&input.history_rows, series_column)?;
    let grouped_future = group_rows(&input.future_rows, series_column)?;
    let mut y = Vec::<f64>::new();
    let mut sizes = Vec::<usize>::new();
    let mut x = Vec::<Vec<Value>>::new();
    let mut x_future = Vec::<Vec<Value>>::new();
    for (series_id, history_rows) in &grouped_history {
        sizes.push(history_rows.len());
        for row in history_rows {
            y.push(read_numeric_field(row, &request.target_column)?);
            if !request.covariate_columns.is_empty() {
                x.push(read_exogenous_row(row, &request.covariate_columns)?);
            }
        }
        if let Some(future_rows) = grouped_future.get(series_id) {
            for row in future_rows {
                if !request.covariate_columns.is_empty() {
                    x_future.push(read_exogenous_row(row, &request.covariate_columns)?);
                }
            }
        }
    }
    let mut series = json!({ "y": y, "sizes": sizes });
    if !request.covariate_columns.is_empty() {
        series["X"] =
            serde_json::to_value(x).map_err(|_| "Données de contexte invalides".to_string())?;
        if !input.future_rows.is_empty() {
            series["X_future"] = serde_json::to_value(x_future)
                .map_err(|_| "Données de contexte futur invalides".to_string())?;
        }
    }
    let model = request.model.as_deref().unwrap_or("timegpt-2-standard");
    let config = client_nixtla_options::effective_config(model);
    let level = client_nixtla_options::effective_level(request.confidence_level);
    let mut payload = json!({
        "series": series,
        "freq": request.frequency,
        "h": request.horizon,
        "model": model,
        "level": [level],
        "clean_ex_first": true,
    });
    client_nixtla_options::apply(&mut payload, &config);
    Ok(payload)
}
pub fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<ParsedNixtla, String> {
    let mean = body["mean"]
        .as_array()
        .ok_or("Réponse Nixtla: champ mean manquant")?;
    let level = client_nixtla_options::effective_level(request.confidence_level);
    let lower = client_nixtla_options::interval_array(body, &format!("lo-{level}"))?;
    let upper = client_nixtla_options::interval_array(body, &format!("hi-{level}"))?;
    let dates = super::input_future::expected_dates_by_series(request, input)?;
    let horizon = request.horizon as usize;
    let expected_count = dates.len().saturating_mul(horizon);
    if mean.len() != expected_count
        || lower.len() != expected_count
        || upper.len() != expected_count
    {
        return Err("Réponse Nixtla incomplète".into());
    }
    let mut predictions = Vec::with_capacity(mean.len());
    let mut q10 = Vec::with_capacity(mean.len());
    let mut q50 = Vec::with_capacity(mean.len());
    let mut q90 = Vec::with_capacity(mean.len());
    let mut offset = 0usize;
    for (series_id, series_dates) in dates {
        for index in 0..horizon {
            let value = mean
                .get(offset)
                .and_then(Value::as_f64)
                .ok_or("Réponse Nixtla: valeur mean invalide")?;
            predictions.push(Prediction {
                date: series_dates
                    .get(index)
                    .cloned()
                    .ok_or("Dates futures invalides")?,
                value,
                series_id: Some(series_id.clone()),
            });
            q10.push(*lower.get(offset).ok_or("Intervalle Nixtla incomplet")?);
            q50.push(value);
            q90.push(*upper.get(offset).ok_or("Intervalle Nixtla incomplet")?);
            offset += 1;
        }
    }
    Ok((predictions, q10, q50, q90))
}
fn group_rows(
    rows: &[Value],
    series_column: &str,
) -> Result<BTreeMap<String, Vec<Map<String, Value>>>, String> {
    let mut grouped = BTreeMap::<String, Vec<Map<String, Value>>>::new();
    for row in rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        let series_value = object
            .get(series_column)
            .ok_or("Valeur de série invalide")?;
        let series_id = super::input_series::normalize_series_value(series_value)?
            .ok_or("Valeur de série invalide")?;
        grouped.entry(series_id).or_default().push(object.clone());
    }
    Ok(grouped)
}

fn read_exogenous_row(row: &Map<String, Value>, columns: &[String]) -> Result<Vec<Value>, String> {
    columns
        .iter()
        .map(|column| match row.get(column) {
            Some(Value::Number(number)) => {
                Ok(Value::from(number.as_f64().ok_or("Covariables invalides")?))
            }
            Some(Value::Bool(flag)) => Ok(Value::from(if *flag { 1.0 } else { 0.0 })),
            Some(Value::Null) | None => Ok(Value::Null),
            Some(_) => Err("Covariables invalides".into()),
        })
        .collect()
}

fn read_numeric_field(row: &Map<String, Value>, column: &str) -> Result<f64, String> {
    row.get(column)
        .and_then(Value::as_f64)
        .ok_or("Colonne cible non numérique".into())
}
