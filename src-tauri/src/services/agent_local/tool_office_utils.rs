use serde_json::Value;
use std::path::{Path, PathBuf};

/// Résout un chemin relatif/absolu/tilde en chemin absolu.
pub fn resolve_path(path: &str, working_dir: &Path) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.strip_prefix("~/").unwrap_or(&path[1..]));
        }
    }
    let raw = Path::new(path);
    if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        working_dir.join(raw)
    }
}

/// Extrait un u32 d'une Value, tolère string ou number.
pub fn value_as_u32(val: &Value) -> Option<u32> {
    if let Some(n) = val.as_u64() {
        return Some(n as u32);
    }
    if let Some(n) = val.as_f64() {
        return Some(n as u32);
    }
    val.as_str().and_then(|s| s.trim().parse::<u32>().ok())
}

/// Extrait un u16 d'une Value, tolère string ou number.
pub fn value_as_u16(val: &Value) -> Option<u16> {
    value_as_u32(val).map(|n| n as u16)
}

/// Extrait un f64 d'une Value, tolère string ou number.
pub fn value_as_f64(val: &Value) -> Option<f64> {
    if let Some(n) = val.as_f64() {
        return Some(n);
    }
    val.as_str().and_then(|s| s.trim().parse::<f64>().ok())
}

/// Traduit les noms de fonctions Excel français → anglais.
const FR_FORMULAS: &[(&str, &str)] = &[
    ("SOMME", "SUM"), ("MOYENNE", "AVERAGE"), ("NB", "COUNT"),
    ("NBVAL", "COUNTA"), ("NB.SI", "COUNTIF"), ("NB.VIDE", "COUNTBLANK"),
    ("SOMME.SI", "SUMIF"), ("SOMME.SI.ENS", "SUMIFS"),
    ("MAX", "MAX"), ("MIN", "MIN"), ("SI", "IF"),
    ("ET", "AND"), ("OU", "OR"), ("NON", "NOT"),
    ("RECHERCHEV", "VLOOKUP"), ("RECHERCHEH", "HLOOKUP"),
    ("INDEX", "INDEX"), ("EQUIV", "MATCH"),
    ("CONCATENER", "CONCATENATE"), ("GAUCHE", "LEFT"),
    ("DROITE", "RIGHT"), ("MAJUSCULE", "UPPER"), ("MINUSCULE", "LOWER"),
    ("NBCAR", "LEN"), ("ARRONDI", "ROUND"),
    ("ABS", "ABS"), ("RACINE", "SQRT"), ("PUISSANCE", "POWER"),
    ("AUJOURDHUI", "TODAY"), ("MAINTENANT", "NOW"),
    ("ANNEE", "YEAR"), ("MOIS", "MONTH"), ("JOUR", "DAY"),
    ("VRAI", "TRUE"), ("FAUX", "FALSE"),
    ("STXT", "MID"), ("CHERCHE", "SEARCH"), ("TROUVE", "FIND"),
    ("ENT", "INT"), ("MOD", "MOD"),
];

/// Normalise une formule : traduit le français, remplace `;` par `,`.
pub fn normalize_formula(formula: &str) -> String {
    let mut result = formula.to_string();
    for &(fr, en) in FR_FORMULAS {
        if fr == en { continue; }
        let pattern = format!("{}(", fr);
        let replacement = format!("{}(", en);
        result = result.replace(&pattern, &replacement);
    }
    result.replace(';', ",")
}

/// Coerce une Value en Vec<Value> pour les arrays de `values` dans set_row.
pub fn coerce_values_array(val: &Value) -> Option<Vec<Value>> {
    if let Some(arr) = val.as_array() {
        return Some(arr.clone());
    }
    if let Some(s) = val.as_str() {
        if let Ok(parsed) = serde_json::from_str::<Value>(s.trim()) {
            if let Some(arr) = parsed.as_array() {
                return Some(arr.clone());
            }
        }
    }
    None
}
