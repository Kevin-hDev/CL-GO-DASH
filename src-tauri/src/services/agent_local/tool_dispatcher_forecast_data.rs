use serde_json::Value;
use std::path::Path;

pub async fn load_file_data(raw_path: &str, working_dir: &Path) -> Result<String, String> {
    let result = super::tool_spreadsheet_read::read_spreadsheet(
        raw_path,
        None,
        None,
        Some(5000),
        working_dir,
    )
    .await;
    if result.is_error {
        return Err(result.content);
    }
    spreadsheet_to_records_json(&result.content)
}

fn spreadsheet_to_records_json(content: &str) -> Result<String, String> {
    let parsed: Value =
        serde_json::from_str(content).map_err(|_| "Données de fichier invalides".to_string())?;
    let headers = parsed["headers"].as_array().ok_or("En-têtes manquants")?;
    let rows = parsed["rows"].as_array().ok_or("Lignes manquantes")?;
    let header_names: Vec<String> = headers
        .iter()
        .filter_map(|h| h.as_str())
        .filter(|h| !h.trim().is_empty())
        .map(|h| h.to_string())
        .collect();
    if header_names.is_empty() {
        return Err("En-têtes manquants".into());
    }
    let records: Vec<Value> = rows
        .iter()
        .filter_map(|row| row.as_array())
        .map(|row| {
            let mut obj = serde_json::Map::with_capacity(header_names.len());
            for (idx, name) in header_names.iter().enumerate() {
                obj.insert(
                    name.clone(),
                    normalize_cell(row.get(idx).cloned().unwrap_or(Value::Null)),
                );
            }
            Value::Object(obj)
        })
        .collect();
    serde_json::to_string(&records).map_err(|_| "Conversion fichier impossible".into())
}

fn normalize_cell(value: Value) -> Value {
    let Some(raw) = value.as_str() else {
        return value;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Value::String(String::new());
    }
    let normalized = trimmed.replace(',', ".");
    if let Ok(number) = normalized.parse::<f64>() {
        if number.is_finite() {
            return serde_json::Number::from_f64(number).map_or(Value::Null, Value::Number);
        }
    }
    Value::String(trimmed.to_string())
}
