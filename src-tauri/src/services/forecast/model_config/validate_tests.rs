//! Tests de sanitize (PURE) : validation et normalisation des paramètres
//! forecast typés (Integer/Number/Boolean/Select/NumberList).

use super::*;
use crate::services::forecast::model_config::schema::{ParamKind, ParamSpec};
use serde_json::{json, Map};

/// Construit un ParamSpec statique pour les tests.
fn spec(id: &'static str, kind: ParamKind) -> ParamSpec {
    ParamSpec {
        id,
        kind,
        default_value: json!(null),
        min: None,
        max: None,
        options: &[],
    }
}

fn spec_with_range(id: &'static str, kind: ParamKind, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        id,
        kind,
        default_value: json!(null),
        min: Some(min),
        max: Some(max),
        options: &[],
    }
}

// --- Integer ---------------------------------------------------------------

#[test]
fn integer_accepts_valid_value() {
    let specs = [spec("horizon", ParamKind::Integer)];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(12));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["horizon"], 12);
}

#[test]
fn integer_accepts_string_numeric() {
    // Tolérance : "12" est parsé en entier.
    let specs = [spec("horizon", ParamKind::Integer)];
    let mut values = Map::new();
    values.insert("horizon".into(), json!("12"));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["horizon"], 12);
}

#[test]
fn integer_rejects_out_of_range() {
    let specs = [spec_with_range("horizon", ParamKind::Integer, 1.0, 100.0)];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(200));

    assert!(sanitize(&specs, values).is_err());
}

// --- Number ----------------------------------------------------------------

#[test]
fn number_accepts_float() {
    let specs = [spec("lr", ParamKind::Number)];
    let mut values = Map::new();
    values.insert("lr".into(), json!(0.001));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["lr"], 0.001);
}

#[test]
fn number_rejects_non_finite() {
    // NaN / Infinity ne sont pas valides.
    let specs = [spec("lr", ParamKind::Number)];
    let mut values = Map::new();
    values.insert("lr".into(), json!(f64::NAN));

    // serde_json ne sérialise pas NaN, on teste via string.
    values.clear();
    values.insert("lr".into(), json!("NaN"));
    assert!(sanitize(&specs, values).is_err());
}

// --- Boolean ---------------------------------------------------------------

#[test]
fn boolean_accepts_true_false() {
    let specs = [spec("flag", ParamKind::Boolean)];

    let mut values_true = Map::new();
    values_true.insert("flag".into(), json!(true));
    let result_true = sanitize(&specs, values_true).unwrap();
    assert_eq!(result_true["flag"], true);

    let mut values_false = Map::new();
    values_false.insert("flag".into(), json!(false));
    let result_false = sanitize(&specs, values_false).unwrap();
    assert_eq!(result_false["flag"], false);
}

#[test]
fn boolean_rejects_non_bool() {
    let specs = [spec("flag", ParamKind::Boolean)];
    let mut values = Map::new();
    values.insert("flag".into(), json!("true")); // string, pas bool

    assert!(sanitize(&specs, values).is_err());
}

// --- Select ----------------------------------------------------------------

#[test]
fn select_accepts_valid_option() {
    let specs = [ParamSpec {
        id: "mode",
        kind: ParamKind::Select,
        default_value: json!("a"),
        min: None,
        max: None,
        options: &["a", "b", "c"],
    }];
    let mut values = Map::new();
    values.insert("mode".into(), json!("b"));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["mode"], "b");
}

#[test]
fn select_rejects_invalid_option() {
    let specs = [ParamSpec {
        id: "mode",
        kind: ParamKind::Select,
        default_value: json!("a"),
        min: None,
        max: None,
        options: &["a", "b"],
    }];
    let mut values = Map::new();
    values.insert("mode".into(), json!("z"));

    assert!(sanitize(&specs, values).is_err());
}

// --- NumberList ------------------------------------------------------------

#[test]
fn number_list_accepts_array() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!([80.0, 90.0, 95.0]));

    let result = sanitize(&specs, values).unwrap();
    let arr = result["levels"].as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn number_list_sorts_and_dedup() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!([95.0, 80.0, 90.0, 80.0]));

    let result = sanitize(&specs, values).unwrap();
    let arr: Vec<f64> = result["levels"].as_array().unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert_eq!(arr, vec![80.0, 90.0, 95.0]); // trié, dédupliqué
}

#[test]
fn number_list_rejects_too_many() {
    // MAX_NUMBER_LIST = 9.
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]));

    assert!(sanitize(&specs, values).is_err());
}

#[test]
fn number_list_accepts_comma_separated_string() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!("80, 90, 95"));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["levels"].as_array().unwrap().len(), 3);
}

// --- Comportements globaux -------------------------------------------------

#[test]
fn null_and_empty_values_are_removed() {
    let specs = [spec("a", ParamKind::Integer), spec("b", ParamKind::Select)];
    let mut values = Map::new();
    values.insert("a".into(), Value::Null);
    values.insert("b".into(), json!("   ")); // whitespace only

    let result = sanitize(&specs, values).unwrap();
    assert!(result.is_empty(), "null et string vide doivent être retirés");
}

#[test]
fn unknown_param_id_rejected() {
    let specs = [spec("known", ParamKind::Integer)];
    let mut values = Map::new();
    values.insert("unknown".into(), json!(1));

    assert!(sanitize(&specs, values).is_err());
}

#[test]
fn default_value_not_included_in_output() {
    // Une valeur égale au default n'est pas écrite dans le résultat.
    let specs = [ParamSpec {
        id: "horizon",
        kind: ParamKind::Integer,
        default_value: json!(10),
        min: None,
        max: None,
        options: &[],
    }];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(10));

    let result = sanitize(&specs, values).unwrap();
    assert!(result.is_empty(), "la valeur par défaut ne doit pas être écrite");
}

#[test]
fn non_default_value_included_in_output() {
    let specs = [ParamSpec {
        id: "horizon",
        kind: ParamKind::Integer,
        default_value: json!(10),
        min: None,
        max: None,
        options: &[],
    }];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(20));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["horizon"], 20);
}
