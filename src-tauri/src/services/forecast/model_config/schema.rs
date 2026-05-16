use serde::Serialize;
use serde_json::Value;

use super::params::*;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    Integer,
    Number,
    Boolean,
    Select,
    NumberList,
}

#[derive(Clone)]
pub struct ParamSpec {
    pub id: &'static str,
    pub kind: ParamKind,
    pub default_value: Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub options: &'static [&'static str],
}

pub fn specs_for_family(family_id: &str) -> Vec<ParamSpec> {
    match family_id {
        "chronos-bolt" => vec![
            horizon_override(),
            context(),
            quantiles(),
            precision(),
            batch_size(),
        ],
        "chronos-2" => vec![
            horizon_override(),
            context(),
            quantiles(),
            precision(),
            batch_size(),
        ],
        "timesfm-2-5" => vec![
            horizon_override(),
            context(),
            horizon_len(),
            bool_param("normalization", true),
            bool_param("quantile_head", true),
            select("xreg_mode", "off", &["off", "auto"]),
        ],
        "timegpt-2" => vec![
            horizon_override(),
            confidence(),
            level(),
            bool_param("clean_ex_first", true),
            int_param("finetune_steps", 0, 0.0, 1000.0),
            select(
                "finetune_loss",
                "default",
                &["default", "mae", "mse", "rmse"],
            ),
            int_param("finetune_depth", 0, 0.0, 10.0),
            bool_param("feature_contributions", false),
        ],
        "toto-2" => vec![
            horizon_override(),
            int_param("decode_block_size", 768, 1.0, 4096.0),
            bool_param("has_missing_values", false),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "moirai-2" => vec![
            horizon_override(),
            context(),
            batch_size(),
            int_param("target_dim", 1, 1.0, 256.0),
            int_param("feat_dynamic_real_dim", 0, 0.0, 256.0),
            int_param("past_feat_dynamic_real_dim", 0, 0.0, 256.0),
        ],
        "flowstate" => vec![
            horizon_override(),
            int_param("prediction_length", 0, 0.0, 100_000.0),
            number_param("scale_factor", 1.0, 0.0001, 1000.0),
            bool_param("batch_first", true),
            precision(),
        ],
        "tabpfn-ts" => vec![
            horizon_override(),
            bool_param("known_future_covariates", true),
            select("local_mode", "local", &["local"]),
            bool_param("probabilistic_output", true),
            context(),
        ],
        "tirex" => vec![
            horizon_override(),
            select("backend", "auto", &["auto", "torch"]),
            batch_size(),
            select("output_type", "quantiles", &["quantiles", "mean"]),
            select("frequency_resampling", "auto", &["auto", "off"]),
            quantiles(),
        ],
        "kairos" => vec![
            horizon_override(),
            bool_param("preserve_positivity", true),
            bool_param("average_with_flipped_input", true),
            bool_param("generation", true),
            quantiles(),
        ],
        "sundial" => vec![
            horizon_override(),
            int_param("num_samples", 64, 1.0, 512.0),
            context(),
            dtype(),
            quantiles(),
        ],
        _ => Vec::new(),
    }
}
