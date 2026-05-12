use super::types::ForecastRequest;
use serde_json::Value;
use std::path::Path;

const MAX_SPREADSHEET_SIZE: u64 = 50 * 1024 * 1024;
const HARD_MAX_ROWS: usize = 5000;

pub async fn ensure_request_data(
    request: &mut ForecastRequest,
    base_dir: Option<&Path>,
) -> Result<(), String> {
    let Some(raw_path) = request.file_path.as_deref() else {
        return Ok(());
    };

    if request.data.as_deref().is_some_and(is_usable_json_payload) {
        return Ok(());
    }

    request.data = Some(load_file_data(raw_path, base_dir).await?);
    Ok(())
}

async fn load_file_data(raw_path: &str, base_dir: Option<&Path>) -> Result<String, String> {
    let base = base_dir.and_then(|path| path.to_str());
    let resolved = crate::commands::file_preview::resolve_preview_path(raw_path, base)?;

    let metadata = tokio::fs::metadata(&resolved)
        .await
        .map_err(|_| "Fichier introuvable".to_string())?;
    if !metadata.is_file() {
        return Err("Chemin invalide".into());
    }
    if metadata.len() > MAX_SPREADSHEET_SIZE {
        return Err("Fichier trop volumineux".into());
    }

    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let preview = match ext.as_str() {
        "csv" | "tsv" => {
            crate::services::agent_local::tool_spreadsheet_read::read_csv(&resolved, HARD_MAX_ROWS)
        }
        "xlsx" | "xls" | "ods" | "xlsm" => {
            crate::services::agent_local::tool_spreadsheet_calamine::read_excel(
                &resolved,
                None,
                None,
                HARD_MAX_ROWS,
            )
        }
        _ => Err("Format non supporté".into()),
    }?;

    preview_to_records_json(&preview)
}

fn preview_to_records_json(preview: &Value) -> Result<String, String> {
    let headers = preview["headers"].as_array().ok_or("En-têtes manquants")?;
    let rows = preview["rows"].as_array().ok_or("Lignes manquantes")?;
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

fn is_usable_json_payload(data: &str) -> bool {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return false;
    }
    serde_json::from_str::<Value>(trimmed).is_ok_and(|value| value.is_array())
}
