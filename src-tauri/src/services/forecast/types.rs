use super::input_data::InputSnapshot;
use serde::{Deserialize, Serialize};

pub const MAX_PREDICTIONS: usize = 10_000;
pub const MAX_ANNOTATIONS: usize = 200;
pub const MAX_SCENARIOS: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastRequest {
    pub data: Option<String>,
    pub file_path: Option<String>,
    pub target_column: String,
    pub date_column: String,
    #[serde(default)]
    pub covariate_columns: Vec<String>,
    pub horizon: u32,
    pub frequency: String,
    pub model: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence_level: f64,
}

fn default_confidence() -> f64 {
    0.9
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub target_column: String,
    pub created_at: String,
    pub session_id: Option<String>,
    pub model: String,
    pub provider: String,
    pub horizon: u32,
    pub frequency: String,
    pub input_summary: InputSummary,
    #[serde(default)]
    pub input_data: InputSnapshot,
    pub predictions: Vec<Prediction>,
    pub quantiles: Quantiles,
    #[serde(default)]
    pub covariates_used: Vec<String>,
    pub metrics: Option<ForecastMetrics>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSummary {
    pub points: usize,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub date: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantiles {
    #[serde(default)]
    pub q10: Vec<f64>,
    #[serde(default)]
    pub q50: Vec<f64>,
    #[serde(default)]
    pub q90: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastMetrics {
    pub mape: Option<f64>,
    pub mae: Option<f64>,
    pub crps: Option<f64>,
    pub bias: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub date: String,
    pub text: String,
    pub source: AnnotationSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationSource {
    User,
    Llm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub predictions: Vec<Prediction>,
    pub quantiles: Quantiles,
    pub params_modified: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastAnalysisMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub model: String,
    pub provider: String,
    pub horizon: u32,
    pub frequency: String,
    pub points: usize,
    pub mape: Option<f64>,
    pub session_id: Option<String>,
}

impl ForecastResult {
    pub fn to_meta(&self) -> ForecastAnalysisMeta {
        ForecastAnalysisMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at.clone(),
            model: self.model.clone(),
            provider: self.provider.clone(),
            horizon: self.horizon,
            frequency: self.frequency.clone(),
            points: self.input_summary.points,
            mape: self.metrics.as_ref().and_then(|m| m.mape),
            session_id: self.session_id.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub device: String,
    #[serde(default = "default_quantiles")]
    pub quantiles: Vec<f64>,
    #[serde(default = "default_horizon_max")]
    pub horizon_max: u32,
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    pub precision: String,
}

fn default_quantiles() -> Vec<f64> {
    vec![0.1, 0.5, 0.9]
}

fn default_horizon_max() -> u32 {
    1000
}

fn default_batch_size() -> u32 {
    32
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            device: "auto".into(),
            quantiles: default_quantiles(),
            horizon_max: default_horizon_max(),
            batch_size: default_batch_size(),
            precision: "fp32".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastProgress {
    pub stage: String,
    pub percent: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadProgress {
    pub model_name: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}
