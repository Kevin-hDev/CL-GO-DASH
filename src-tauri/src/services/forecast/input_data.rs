use super::input_parse_utils::{
    build_known_or_relative_dates, build_summary_bounds, collect_columns, read_target_value,
    validate_columns, validate_multiseries_future_rows,
};
use super::input_series::read_series_id;
use super::types::{ForecastRequest, InputSummary, Prediction};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

const MAX_INPUT_ROWS: usize = 5_000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputSnapshot {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub rows: Vec<Value>,
    #[serde(default)]
    pub series_column: Option<String>,
    #[serde(default)]
    pub series_ids: Vec<String>,
    #[serde(default)]
    pub history: Vec<Prediction>,
}

#[derive(Debug, Clone)]
pub struct ParsedInput {
    pub values: Vec<f64>,
    pub future_dates: Vec<String>,
    pub summary: InputSummary,
    pub snapshot: InputSnapshot,
    pub history_rows: Vec<Value>,
    pub future_rows: Vec<Value>,
}

pub fn parse_request_input(request: &ForecastRequest) -> Result<ParsedInput, String> {
    let json_str = request.data.as_ref().ok_or("Données JSON requises")?;
    let rows: Vec<Value> =
        serde_json::from_str(json_str).map_err(|_| "Données JSON invalides".to_string())?;
    if rows.is_empty() {
        return Err("Aucun point de données".into());
    }
    if rows.len() > MAX_INPUT_ROWS {
        return Err("Jeu de données trop volumineux".into());
    }

    let mut columns = Vec::new();
    let mut history = Vec::new();
    let mut values = Vec::new();
    let mut history_rows = Vec::new();
    let mut future_rows = Vec::new();
    let mut future_phase = false;
    let mut future_phase_by_series = HashSet::new();
    let mut history_series_ids = BTreeSet::new();
    let mut series_ids = BTreeSet::new();

    for row in &rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        collect_columns(&mut columns, object.keys())?;
        let series_id = read_series_id(object, request)?;
        let phase_key = series_id.clone().unwrap_or_default();
        if let Some(id) = &series_id {
            series_ids.insert(id.clone());
        }
        let date = object
            .get(&request.date_column)
            .and_then(Value::as_str)
            .ok_or("Colonne date manquante")?;
        match read_target_value(object.get(&request.target_column))? {
            Some(value) => {
                if request.series_column.is_some() {
                    if future_phase_by_series.contains(&phase_key) {
                        return Err("Lignes futures invalides".into());
                    }
                    history_series_ids.insert(phase_key);
                } else if future_phase {
                    return Err("Lignes futures invalides".into());
                }
                history.push(Prediction {
                    date: date.to_string(),
                    value,
                    series_id: series_id.clone(),
                });
                values.push(value);
                history_rows.push(row.clone());
            }
            None => {
                future_phase = true;
                if request.series_column.is_some() {
                    future_phase_by_series.insert(phase_key);
                }
                future_rows.push(row.clone());
            }
        }
    }

    validate_columns(&columns, request)?;
    if history.is_empty() {
        return Err("Aucun point de données historiques".into());
    }

    let future_dates = build_known_or_relative_dates(&history, &future_rows, request)?;
    validate_multiseries_future_rows(&future_rows, &history_series_ids, request)?;
    let (start, end) = build_summary_bounds(&history);

    Ok(ParsedInput {
        values,
        future_dates,
        summary: InputSummary {
            points: history.len(),
            start,
            end,
        },
        snapshot: InputSnapshot {
            columns,
            rows,
            series_column: request.series_column.clone(),
            series_ids: series_ids.into_iter().collect(),
            history,
        },
        history_rows,
        future_rows,
    })
}
