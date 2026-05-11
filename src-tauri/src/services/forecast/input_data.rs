use super::types::{ForecastRequest, InputSummary, Prediction};
use chrono::{Duration, Months, NaiveDate, NaiveDateTime};
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
    let mut history = Vec::with_capacity(rows.len());
    let mut values = Vec::with_capacity(rows.len());

    for row in &rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        for key in object.keys() {
            if !columns.iter().any(|existing| existing == key) {
                columns.push(key.clone());
            }
        }
        if columns.len() > MAX_INPUT_COLUMNS {
            return Err("Trop de colonnes".into());
        }

        let date = object
            .get(&request.date_column)
            .and_then(Value::as_str)
            .ok_or("Colonne date manquante")?;
        let value = read_numeric_value(object.get(&request.target_column))
            .ok_or("Colonne cible non numérique")?;

        history.push(Prediction {
            date: date.to_string(),
            value,
        });
        values.push(value);
    }
    if !columns.iter().any(|column| column == &request.target_column) {
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

    let future_dates = build_future_dates(
        history.last().map(|point| point.date.as_str()).unwrap_or_default(),
        &request.frequency,
        request.horizon,
    );

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
    })
}

fn read_numeric_value(value: Option<&Value>) -> Option<f64> {
    match value {
        Some(Value::Number(number)) => number.as_f64().filter(|numeric| numeric.is_finite()),
        Some(Value::String(raw)) => raw.trim().replace(',', ".").parse::<f64>().ok(),
        _ => None,
    }
}

fn build_future_dates(last_date: &str, frequency: &str, horizon: u32) -> Vec<String> {
    let Some(last_datetime) = parse_datetime(last_date) else {
        return (1..=horizon).map(|index| format!("T+{index}")).collect();
    };
    let normalized = frequency.trim().to_uppercase();

    (1..=horizon)
        .map(|step| {
            shift_datetime(last_datetime, &normalized, step)
                .map(|value| format_output(value, last_date))
                .unwrap_or_else(|| format!("T+{step}"))
        })
        .collect()
}

fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
    ];
    for format in formats {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Some(parsed);
        }
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(value, "%Y/%m/%d"))
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}

fn shift_datetime(base: NaiveDateTime, frequency: &str, step: u32) -> Option<NaiveDateTime> {
    match frequency {
        "S" => Some(base + Duration::seconds(i64::from(step))),
        "T" | "MIN" => Some(base + Duration::minutes(i64::from(step))),
        "H" => Some(base + Duration::hours(i64::from(step))),
        "D" | "B" => Some(base + Duration::days(i64::from(step))),
        "W" => Some(base + Duration::weeks(i64::from(step))),
        "M" => base.checked_add_months(Months::new(step)),
        "Q" => base.checked_add_months(Months::new(step.saturating_mul(3))),
        "Y" | "A" => base.checked_add_months(Months::new(step.saturating_mul(12))),
        _ => parse_compound_frequency(base, frequency, step),
    }
}

fn parse_compound_frequency(
    base: NaiveDateTime,
    frequency: &str,
    step: u32,
) -> Option<NaiveDateTime> {
    let digits_len = frequency
        .chars()
        .take_while(|char| char.is_ascii_digit())
        .count();
    if digits_len == 0 || digits_len >= frequency.len() {
        return None;
    }
    let factor = frequency[..digits_len].parse::<u32>().ok()?;
    let unit = &frequency[digits_len..];
    let total = factor.saturating_mul(step);
    match unit {
        "S" => Some(base + Duration::seconds(i64::from(total))),
        "MIN" | "T" => Some(base + Duration::minutes(i64::from(total))),
        "H" => Some(base + Duration::hours(i64::from(total))),
        "D" => Some(base + Duration::days(i64::from(total))),
        "W" => Some(base + Duration::weeks(i64::from(total))),
        _ => None,
    }
}

fn format_output(value: NaiveDateTime, source: &str) -> String {
    if source.contains('T') {
        return value.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    if source.contains(':') {
        return value.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    value.date().format("%Y-%m-%d").to_string()
}
