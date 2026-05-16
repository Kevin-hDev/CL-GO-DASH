use serde_json::Value;

pub fn lower_array(body: &Value) -> Vec<f64> {
    exact_array(body, "q10").unwrap_or_else(|| bound_array(body, Bound::Lower))
}

pub fn upper_array(body: &Value) -> Vec<f64> {
    exact_array(body, "q90").unwrap_or_else(|| bound_array(body, Bound::Upper))
}

pub fn lower_value(item: &Value) -> Option<f64> {
    item["q10"]
        .as_f64()
        .or_else(|| bound_value(item, Bound::Lower))
}

pub fn upper_value(item: &Value) -> Option<f64> {
    item["q90"]
        .as_f64()
        .or_else(|| bound_value(item, Bound::Upper))
}

fn exact_array(body: &Value, key: &str) -> Option<Vec<f64>> {
    let values = body[key]
        .as_array()?
        .iter()
        .map(Value::as_f64)
        .collect::<Option<Vec<_>>>()?;
    if values.is_empty() {
        return None;
    }
    Some(values)
}

fn bound_array(body: &Value, bound: Bound) -> Vec<f64> {
    let Some((key, _)) = select_bound_key(body, bound) else {
        return Vec::new();
    };
    exact_array(body, key).unwrap_or_default()
}

fn bound_value(item: &Value, bound: Bound) -> Option<f64> {
    let (key, _) = select_bound_key(item, bound)?;
    item[key].as_f64()
}

fn select_bound_key(value: &Value, bound: Bound) -> Option<(&str, u32)> {
    let object = value.as_object()?;
    object
        .keys()
        .filter_map(|key| quantile_level(key).map(|level| (key.as_str(), level)))
        .filter(|(_, level)| match bound {
            Bound::Lower => *level < 50,
            Bound::Upper => *level > 50,
        })
        .min_by_key(|(_, level)| match bound {
            Bound::Lower => *level,
            Bound::Upper => 100 - *level,
        })
}

fn quantile_level(key: &str) -> Option<u32> {
    let raw = key.strip_prefix('q')?;
    if raw.len() != 2 {
        return None;
    }
    let level = raw.parse::<u32>().ok()?;
    if (1..=99).contains(&level) {
        Some(level)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum Bound {
    Lower,
    Upper,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_custom_simple_quantiles_to_visible_bounds() {
        let body = json!({
            "median": [2.0, 3.0],
            "q25": [1.0, 2.0],
            "q50": [2.0, 3.0],
            "q75": [4.0, 5.0]
        });

        assert_eq!(lower_array(&body), vec![1.0, 2.0]);
        assert_eq!(upper_array(&body), vec![4.0, 5.0]);
    }

    #[test]
    fn keeps_standard_quantiles_when_present() {
        let body = json!({
            "q05": [0.0],
            "q10": [1.0],
            "q90": [9.0],
            "q95": [10.0]
        });

        assert_eq!(lower_array(&body), vec![1.0]);
        assert_eq!(upper_array(&body), vec![9.0]);
    }

    #[test]
    fn maps_custom_structured_quantiles_to_visible_bounds() {
        let item = json!({
            "q20": 8.0,
            "q50": 10.0,
            "q80": 12.0
        });

        assert_eq!(lower_value(&item), Some(8.0));
        assert_eq!(upper_value(&item), Some(12.0));
    }
}
