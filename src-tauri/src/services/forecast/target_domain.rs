use super::input_data::ParsedInput;
use super::types::{ForecastRequest, Prediction};

pub fn apply_non_negative_floor(
    request: &ForecastRequest,
    input: &ParsedInput,
    predictions: &mut [Prediction],
    q10: &mut [f64],
    q50: &mut [f64],
    q90: &mut [f64],
) {
    if !requires_non_negative_output(request, input) {
        return;
    }
    for prediction in predictions {
        prediction.value = prediction.value.max(0.0);
    }
    clamp_values(q10);
    clamp_values(q50);
    clamp_values(q90);
}

pub fn requires_non_negative_output(request: &ForecastRequest, input: &ParsedInput) -> bool {
    enabled_by_config(request) && is_non_negative_target(request, input)
}

fn enabled_by_config(request: &ForecastRequest) -> bool {
    let Some(model_id) = request.model.as_deref() else {
        return false;
    };
    super::model_config::effective_values(model_id)
        .ok()
        .and_then(|values| values.get("non_negative_output").and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

fn is_non_negative_target(request: &ForecastRequest, input: &ParsedInput) -> bool {
    input
        .values
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
        && target_name_is_non_negative(&request.target_column)
}

fn target_name_is_non_negative(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    if contains_any(
        &normalized,
        &[
            "return",
            "rendement",
            "pct",
            "percent",
            "ratio",
            "margin",
            "marge",
            "delta",
            "variation",
            "temp",
            "temperature",
            "score",
            "indice",
            "index",
        ],
    ) {
        return false;
    }
    contains_any(
        &normalized,
        &[
            "ca", "eur", "revenue", "revenu", "sales", "vente", "amount", "montant", "price",
            "prix", "total", "commande", "order", "count", "qty", "quantite", "quantity", "volume",
        ],
    )
}

fn contains_any(value: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|token| value.contains(token))
}

fn clamp_values(values: &mut [f64]) {
    for value in values {
        *value = value.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_revenue_bounds_but_not_return_metrics() {
        assert!(target_name_is_non_negative("ca_total_eur"));
        assert!(!target_name_is_non_negative("nasdaq_return_pct"));
    }
}
