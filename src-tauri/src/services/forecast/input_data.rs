use super::input_dates::build_future_dates;
use super::types::{ForecastRequest, InputSummary, Prediction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_INPUT_ROWS: usize = 5_000;
const MAX_INPUT_COLUMNS: usize = 256;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputSnapshot {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub rows: Vec<Value>,
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

    for row in &rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        collect_columns(&mut columns, object.keys())?;
        let date = object
            .get(&request.date_column)
            .and_then(Value::as_str)
            .ok_or("Colonne date manquante")?;
        match read_target_value(object.get(&request.target_column))? {
            Some(value) => {
                if future_phase {
                    return Err("Lignes futures invalides".into());
                }
                history.push(Prediction {
                    date: date.to_string(),
                    value,
                });
                values.push(value);
                history_rows.push(row.clone());
            }
            None => {
                future_phase = true;
                future_rows.push(row.clone());
            }
        }
    }

    validate_columns(&columns, request)?;
    if history.is_empty() {
        return Err("Aucun point de données historiques".into());
    }

    let future_dates = build_known_or_relative_dates(&history, &future_rows, request)?;

    Ok(ParsedInput {
        values,
        future_dates,
        summary: InputSummary {
            points: history.len(),
            start: history
                .first()
                .map(|point| point.date.clone())
                .unwrap_or_default(),
            end: history
                .last()
                .map(|point| point.date.clone())
                .unwrap_or_default(),
        },
        snapshot: InputSnapshot {
            columns,
            rows,
            history,
        },
        history_rows,
        future_rows,
    })
}

fn collect_columns<'a, I>(columns: &mut Vec<String>, keys: I) -> Result<(), String>
where
    I: Iterator<Item = &'a String>,
{
    for key in keys {
        if !columns.iter().any(|existing| existing == key) {
            columns.push(key.clone());
        }
    }
    if columns.len() > MAX_INPUT_COLUMNS {
        return Err("Trop de colonnes".into());
    }
    Ok(())
}

fn validate_columns(columns: &[String], request: &ForecastRequest) -> Result<(), String> {
    if !columns
        .iter()
        .any(|column| column == &request.target_column)
    {
        return Err("Colonne cible introuvable".into());
    }
    if !columns.iter().any(|column| column == &request.date_column) {
        return Err("Colonne date introuvable".into());
    }
    for covariate in &request.covariate_columns {
        if !columns.iter().any(|column| column == covariate) {
            return Err("Covariable introuvable".into());
        }
    }
    Ok(())
}

fn read_target_value(value: Option<&Value>) -> Result<Option<f64>, String> {
    match value {
        Some(Value::Number(number)) => Ok(number.as_f64().filter(|numeric| numeric.is_finite())),
        Some(Value::String(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
                .replace(',', ".")
                .parse::<f64>()
                .map(Some)
                .map_err(|_| "Colonne cible non numérique".to_string())
        }
        Some(Value::Null) | None => Ok(None),
        _ => Err("Colonne cible non numérique".into()),
    }
}

fn build_known_or_relative_dates(
    history: &[Prediction],
    future_rows: &[Value],
    request: &ForecastRequest,
) -> Result<Vec<String>, String> {
    if future_rows.is_empty() {
        let last_date = history
            .last()
            .map(|point| point.date.as_str())
            .unwrap_or_default();
        return Ok(build_future_dates(
            last_date,
            &request.frequency,
            request.horizon,
        ));
    }
    if future_rows.len() != request.horizon as usize {
        return Err("Nombre de lignes futures invalide".into());
    }

    future_rows
        .iter()
        .map(|row| {
            row[&request.date_column]
                .as_str()
                .map(|value| value.to_string())
                .ok_or("Colonne date manquante".to_string())
        })
        .collect()
}
