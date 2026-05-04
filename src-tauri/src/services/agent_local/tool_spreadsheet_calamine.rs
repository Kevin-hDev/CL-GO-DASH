use calamine::{Data, Reader};
use serde_json::Value;
use std::path::Path;

fn cell_or_formula(
    cell: &Data,
    row: usize,
    col: usize,
    formulas: Option<&calamine::Range<String>>,
) -> Value {
    if let Some(f_range) = formulas {
        if let Some(formula) = f_range.get((row, col)) {
            if !formula.is_empty() {
                return Value::String(format!("={formula}"));
            }
        }
    }
    cell_to_json(cell)
}

pub fn cell_to_json(cell: &Data) -> Value {
    match cell {
        Data::Empty => Value::Null,
        Data::String(s) => Value::String(s.clone()),
        Data::Float(f) => serde_json::json!(f),
        Data::Int(i) => serde_json::json!(i),
        Data::Bool(b) => Value::Bool(*b),
        Data::DateTime(dt) => Value::String(format!("{}", dt.as_f64())),
        Data::DateTimeIso(s) => Value::String(s.clone()),
        Data::DurationIso(s) => Value::String(s.clone()),
        Data::Error(e) => Value::String(format!("#ERR:{:?}", e)),
    }
}

pub fn read_excel(
    resolved: &Path,
    sheet: Option<&str>,
    range_str: Option<&str>,
    max_rows: usize,
) -> Result<Value, String> {
    let mut workbook: calamine::Sheets<_> =
        calamine::open_workbook_auto(resolved).map_err(|_| "Impossible d'ouvrir le fichier".to_string())?;

    let sheet_names = workbook.sheet_names().to_owned();
    if sheet_names.is_empty() {
        return Err("Le fichier ne contient aucune feuille".into());
    }

    let effective_sheet = match sheet {
        Some(name) if !name.trim().is_empty() => Some(name),
        _ => None,
    };

    let sheet_name = match effective_sheet {
        Some(name) => {
            if sheet_names.contains(&name.to_string()) {
                name.to_string()
            } else {
                return Err(format!("Feuille '{}' introuvable", name));
            }
        }
        None => sheet_names[0].clone(),
    };

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| "Impossible de lire la feuille".to_string())?;

    let formulas = workbook.worksheet_formula(&sheet_name).ok();

    let bounds = super::tool_spreadsheet_read::parse_range(range_str.unwrap_or(""));
    let all_rows: Vec<Vec<Value>> = range
        .rows()
        .enumerate()
        .filter_map(|(row_idx, row): (usize, &[Data])| {
            if let Some((rs, cs, re, ce)) = bounds {
                if row_idx < rs || row_idx > re {
                    return None;
                }
                let filtered: Vec<Value> = row
                    .iter()
                    .enumerate()
                    .filter(|(col_idx, _)| *col_idx >= cs && *col_idx <= ce)
                    .map(|(col_idx, cell)| {
                        cell_or_formula(cell, row_idx, col_idx, formulas.as_ref())
                    })
                    .collect();
                Some(filtered)
            } else {
                Some(row.iter().enumerate().map(|(col_idx, cell)| {
                    cell_or_formula(cell, row_idx, col_idx, formulas.as_ref())
                }).collect())
            }
        })
        .collect();

    super::tool_spreadsheet_read::build_result(all_rows, max_rows, &sheet_name, &sheet_names)
}
