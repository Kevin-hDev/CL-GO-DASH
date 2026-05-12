use super::input_dates::build_future_dates;
use super::input_data::ParsedInput;
use super::types::{ForecastRequest, Prediction};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    let series_column = request.series_column.as_deref().ok_or("Colonne série manquante")?;
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
        series["X"] = serde_json::to_value(x).map_err(|_| "Données de contexte invalides".to_string())?;
        if !input.future_rows.is_empty() {
            series["X_future"] = serde_json::to_value(x_future)
                .map_err(|_| "Données de contexte futur invalides".to_string())?;
        }
    }

    Ok(json!({
        "series": series,
        "freq": request.frequency,
        "h": request.horizon,
        "model": request.model.as_deref().unwrap_or("timegpt-2-standard"),
        "level": [(request.confidence_level * 100.0) as u32],
        "clean_ex_first": true,
    }))
}

pub fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<(Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
    let mean = body["mean"].as_array().ok_or("Réponse Nixtla: champ mean manquant")?;
    let level = (request.confidence_level * 100.0) as u32;
    let lower = read_interval(body, &format!("lo-{level}"), mean.len());
    let upper = read_interval(body, &format!("hi-{level}"), mean.len());
    let dates = build_prediction_dates(input, request)?;
    let horizon = request.horizon as usize;
    let mut predictions = Vec::with_capacity(mean.len());
    let mut q10 = Vec::with_capacity(mean.len());
    let mut q50 = Vec::with_capacity(mean.len());
    let mut q90 = Vec::with_capacity(mean.len());
    let mut offset = 0usize;

    for (series_id, series_dates) in dates {
        for index in 0..horizon {
            let value = mean.get(offset).and_then(Value::as_f64).ok_or("Réponse Nixtla: valeur mean invalide")?;
            predictions.push(Prediction {
                date: series_dates.get(index).cloned().unwrap_or_else(|| format!("T+{}", index + 1)),
                value,
                series_id: Some(series_id.clone()),
            });
            q10.push(lower.get(offset).copied().unwrap_or(value));
            q50.push(value);
            q90.push(upper.get(offset).copied().unwrap_or(value));
            offset += 1;
        }
    }

    Ok((predictions, q10, q50, q90))
}

fn group_rows(rows: &[Value], series_column: &str) -> Result<BTreeMap<String, Vec<Map<String, Value>>>, String> {
    let mut grouped = BTreeMap::<String, Vec<Map<String, Value>>>::new();
    for row in rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        let series_id = object
            .get(series_column)
            .and_then(Value::as_str)
            .ok_or("Valeur de série invalide")?
            .trim()
            .to_string();
        if series_id.is_empty() {
            return Err("Valeur de série invalide".into());
        }
        grouped.entry(series_id).or_default().push(object.clone());
    }
    Ok(grouped)
}

fn build_prediction_dates(
    input: &ParsedInput,
    request: &ForecastRequest,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let series_column = request.series_column.as_deref().ok_or("Colonne série manquante")?;
    if !input.future_rows.is_empty() {
        return group_rows(&input.future_rows, series_column)?
            .into_iter()
            .map(|(series_id, rows)| {
                rows.iter()
                    .map(|row| {
                        row.get(&request.date_column)
                            .and_then(Value::as_str)
                            .map(|value| value.to_string())
                            .ok_or("Colonne date manquante".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|dates| (series_id, dates))
            })
            .collect();
    }

    Ok(group_rows(&input.history_rows, series_column)?
        .into_iter()
        .map(|(series_id, rows)| {
            let last_date = rows
                .last()
                .and_then(|row| row.get(&request.date_column))
                .and_then(Value::as_str)
                .unwrap_or_default();
            (series_id, build_future_dates(last_date, &request.frequency, request.horizon))
        })
        .collect())
}

fn read_exogenous_row(row: &Map<String, Value>, columns: &[String]) -> Result<Vec<Value>, String> {
    columns
        .iter()
        .map(|column| match row.get(column) {
            Some(Value::Number(number)) => Ok(Value::from(number.as_f64().ok_or("Covariables invalides")?)),
            Some(Value::Bool(flag)) => Ok(Value::from(if *flag { 1.0 } else { 0.0 })),
            Some(Value::Null) | None => Ok(Value::Null),
            Some(_) => Err("Covariables invalides".into()),
        })
        .collect()
}

fn read_numeric_field(row: &Map<String, Value>, column: &str) -> Result<f64, String> {
    row.get(column).and_then(Value::as_f64).ok_or("Colonne cible non numérique".into())
}

fn read_interval(body: &Value, key: &str, fallback_len: usize) -> Vec<f64> {
    body["intervals"][key]
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_else(|| vec![0.0; fallback_len])
}
