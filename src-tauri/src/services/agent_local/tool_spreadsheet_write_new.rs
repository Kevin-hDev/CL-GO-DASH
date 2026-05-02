use crate::services::agent_local::tool_office_utils::{
    coerce_values_array, normalize_formula, value_as_f64, value_as_u16, value_as_u32,
};
use crate::services::agent_local::tool_spreadsheet_write::parse_cell_ref;
use rust_xlsxwriter::Workbook;
use serde_json::Value;
use std::path::Path;

pub fn create_xlsx(path: &Path, ops: &[Value]) -> Result<(), String> {
    let mut workbook = Workbook::new();

    let extra_sheets: Vec<&str> = ops
        .iter()
        .filter(|op| op["type"].as_str() == Some("add_sheet"))
        .filter_map(|op| op["name"].as_str())
        .collect();

    let worksheet = workbook.add_worksheet();

    for op in ops {
        let op_type = op["type"].as_str().unwrap_or("");
        match op_type {
            "set_cell" => apply_set_cell(worksheet, op)?,
            "set_formula" => apply_set_formula(worksheet, op)?,
            "set_row" => apply_set_row(worksheet, op)?,
            "set_column_width" => apply_set_column_width(worksheet, op)?,
            "add_sheet" => {}
            _ => {}
        }
    }

    for name in extra_sheets {
        let ws = workbook.add_worksheet();
        ws.set_name(name)
            .map_err(|_| "Erreur création feuille".to_string())?;
    }

    workbook
        .save(path)
        .map_err(|_| "Impossible de sauvegarder le fichier xlsx".to_string())
}

fn apply_set_cell(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    write_value(ws, row, col, &op["value"])
}

fn apply_set_formula(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    let raw = op["formula"].as_str().unwrap_or("");
    let formula = normalize_formula(raw);
    ws.write_formula(row, col, formula.as_str())
        .map(|_| ())
        .map_err(|_| "Erreur écriture formule".to_string())
}

fn apply_set_row(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let row = value_as_u32(&op["row"]).unwrap_or(0);
    let values = match coerce_values_array(&op["values"]) {
        Some(v) => v,
        None => return Ok(()),
    };
    for (col_idx, val) in values.iter().enumerate() {
        write_value(ws, row, col_idx as u16, val)?;
    }
    Ok(())
}

fn apply_set_column_width(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let col = value_as_u16(&op["col"]).unwrap_or(0);
    let width = value_as_f64(&op["width"]).unwrap_or(8.43);
    ws.set_column_width(col, width)
        .map(|_| ())
        .map_err(|_| "Erreur largeur colonne".to_string())
}

fn resolve_cell_position(op: &Value) -> Result<(u32, u16), String> {
    if let Some(cell_ref) = op["cell"].as_str() {
        parse_cell_ref(cell_ref).ok_or_else(|| "Référence de cellule invalide".to_string())
    } else {
        let row = value_as_u32(&op["row"]).unwrap_or(0);
        let col = value_as_u16(&op["col"]).unwrap_or(0);
        Ok((row, col))
    }
}

pub fn write_value(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    val: &Value,
) -> Result<(), String> {
    match val {
        Value::String(s) => {
            if s.starts_with('=') {
                let formula = normalize_formula(s);
                ws.write_formula(row, col, formula.as_str())
            } else if let Ok(n) = s.parse::<f64>() {
                ws.write_number(row, col, n)
            } else if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
                ws.write_boolean(row, col, s.eq_ignore_ascii_case("true"))
            } else {
                ws.write_string(row, col, s)
            }
        }
        Value::Number(n) => ws.write_number(row, col, n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => ws.write_boolean(row, col, *b),
        Value::Null => return Ok(()),
        _ => ws.write_string(row, col, &val.to_string()),
    }
    .map(|_| ())
    .map_err(|_| "Erreur écriture cellule".to_string())
}
