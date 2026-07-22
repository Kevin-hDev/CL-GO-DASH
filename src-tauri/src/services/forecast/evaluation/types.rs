use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    pub analysis_id: String,
    #[serde(default)]
    pub model_ids: Vec<String>,
    #[serde(default)]
    pub max_windows: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastEvaluation {
    pub schema_version: u32,
    pub created_at: String,
    pub horizon: usize,
    pub windows: usize,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub results: Vec<ModelBacktestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBacktestResult {
    pub model_id: String,
    pub kind: BacktestKind,
    pub metrics: Option<BacktestMetrics>,
    pub calibration: Option<IntervalCalibration>,
    #[serde(default)]
    pub folds: Vec<BacktestFoldMetric>,
    pub duration_ms: u64,
    pub rank: Option<usize>,
    pub beats_best_baseline: Option<bool>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacktestKind {
    Baseline,
    Model,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub mase: f64,
    pub smape: f64,
    pub mae: f64,
    pub rmse: f64,
    pub bias: f64,
    pub stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalCalibration {
    pub theoretical_coverage: f64,
    pub measured_coverage: f64,
    pub mean_width: f64,
    pub residual_half_width: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestFoldMetric {
    pub index: usize,
    pub train_points: usize,
    pub test_points: usize,
    pub mae: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestIndexSummary {
    pub created_at: String,
    pub horizon: usize,
    pub windows: usize,
    #[serde(default)]
    pub results: Vec<BacktestIndexResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestIndexResult {
    pub model_id: String,
    pub kind: BacktestKind,
    pub metrics: Option<BacktestMetrics>,
    pub calibration: Option<IntervalCalibration>,
    pub duration_ms: u64,
    pub beats_best_baseline: Option<bool>,
}

impl ForecastEvaluation {
    pub fn to_index_summary(&self) -> BacktestIndexSummary {
        BacktestIndexSummary {
            created_at: self.created_at.clone(),
            horizon: self.horizon,
            windows: self.windows,
            results: self
                .results
                .iter()
                .map(|result| BacktestIndexResult {
                    model_id: result.model_id.clone(),
                    kind: result.kind,
                    metrics: result.metrics.clone(),
                    calibration: result.calibration.clone(),
                    duration_ms: result.duration_ms,
                    beats_best_baseline: result.beats_best_baseline,
                })
                .collect(),
        }
    }
}
