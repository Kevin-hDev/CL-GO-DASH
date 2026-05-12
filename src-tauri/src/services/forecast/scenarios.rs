use super::{
    storage,
    types::{ForecastResult, Quantiles, Scenario, MAX_SCENARIOS},
};
use serde::{Deserialize, Serialize};

const MAX_SCENARIO_NAME_LEN: usize = 80;
const MAX_SCENARIO_DESCRIPTION_LEN: usize = 500;
const MIN_ADJUSTMENT_PERCENT: f64 = -95.0;
const MAX_ADJUSTMENT_PERCENT: f64 = 500.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioRequest {
    pub analysis_id: String,
    pub name: String,
    pub description: Option<String>,
    pub adjustment_percent: f64,
}

pub async fn create(request: ScenarioRequest) -> Result<ForecastResult, String> {
    validate_request(&request)?;
    let mut analysis = storage::load(&request.analysis_id).await?;
    if analysis.scenarios.len() >= MAX_SCENARIOS {
        return Err("Trop de scénarios".into());
    }

    let factor = 1.0 + request.adjustment_percent / 100.0;
    let scenario = Scenario {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name.trim().to_string(),
        description: clean_description(request.description),
        predictions: analysis
            .predictions
            .iter()
            .map(|point| {
                let mut adjusted = point.clone();
                adjusted.value *= factor;
                adjusted
            })
            .collect(),
        quantiles: scale_quantiles(&analysis.quantiles, factor),
        params_modified: serde_json::json!({
            "kind": "percent_adjustment",
            "adjustment_percent": request.adjustment_percent,
        }),
    };

    analysis.scenarios.push(scenario);
    storage::save(&analysis).await?;
    Ok(analysis)
}

fn validate_request(request: &ScenarioRequest) -> Result<(), String> {
    let name = request.name.trim();
    if name.is_empty() || name.len() > MAX_SCENARIO_NAME_LEN {
        return Err("Nom de scénario invalide".into());
    }
    if let Some(description) = request.description.as_ref() {
        if description.len() > MAX_SCENARIO_DESCRIPTION_LEN {
            return Err("Description de scénario invalide".into());
        }
    }
    if !request.adjustment_percent.is_finite()
        || !(MIN_ADJUSTMENT_PERCENT..=MAX_ADJUSTMENT_PERCENT)
            .contains(&request.adjustment_percent)
    {
        return Err("Ajustement de scénario invalide".into());
    }
    Ok(())
}

fn clean_description(description: Option<String>) -> Option<String> {
    description
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn scale_quantiles(quantiles: &Quantiles, factor: f64) -> Quantiles {
    Quantiles {
        q10: scale_values(&quantiles.q10, factor),
        q50: scale_values(&quantiles.q50, factor),
        q90: scale_values(&quantiles.q90, factor),
    }
}

fn scale_values(values: &[f64], factor: f64) -> Vec<f64> {
    values.iter().map(|value| value * factor).collect()
}
