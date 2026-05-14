use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastEngineKind {
    LocalChronosBolt,
    LocalChronos2,
    CloudApi,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ForecastCapabilities {
    pub past_covariates: bool,
    pub future_covariates: bool,
    pub multivariate: bool,
    pub probabilistic: bool,
    pub backtesting_ready: bool,
    pub anomalies_ready: bool,
    pub fine_tuning_ready: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ForecastRuntimeSpec {
    pub model_id: &'static str,
    pub family_id: &'static str,
    pub engine_kind: ForecastEngineKind,
    pub capabilities: ForecastCapabilities,
}

pub fn find_runtime(model_id: &str) -> Option<&'static ForecastRuntimeSpec> {
    FORECAST_RUNTIMES
        .iter()
        .find(|runtime| runtime.model_id == model_id)
}

pub const FORECAST_RUNTIMES: &[ForecastRuntimeSpec] = &[
    ForecastRuntimeSpec {
        model_id: "chronos-bolt-tiny",
        family_id: "chronos-bolt",
        engine_kind: ForecastEngineKind::LocalChronosBolt,
        capabilities: simple_local_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "chronos-bolt-mini",
        family_id: "chronos-bolt",
        engine_kind: ForecastEngineKind::LocalChronosBolt,
        capabilities: simple_local_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "chronos-bolt-small",
        family_id: "chronos-bolt",
        engine_kind: ForecastEngineKind::LocalChronosBolt,
        capabilities: simple_local_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "chronos-bolt-base",
        family_id: "chronos-bolt",
        engine_kind: ForecastEngineKind::LocalChronosBolt,
        capabilities: simple_local_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "chronos-2",
        family_id: "chronos-2",
        engine_kind: ForecastEngineKind::LocalChronos2,
        capabilities: chronos2_current_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "timegpt-2-mini",
        family_id: "timegpt-2",
        engine_kind: ForecastEngineKind::CloudApi,
        capabilities: advanced_cloud_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "timegpt-2-standard",
        family_id: "timegpt-2",
        engine_kind: ForecastEngineKind::CloudApi,
        capabilities: advanced_cloud_capabilities(),
    },
    ForecastRuntimeSpec {
        model_id: "timegpt-2-pro",
        family_id: "timegpt-2",
        engine_kind: ForecastEngineKind::CloudApi,
        capabilities: advanced_cloud_capabilities(),
    },
];

const fn simple_local_capabilities() -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates: false,
        future_covariates: false,
        multivariate: false,
        probabilistic: true,
        backtesting_ready: false,
        anomalies_ready: false,
        fine_tuning_ready: false,
    }
}

const fn chronos2_current_capabilities() -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates: true,
        future_covariates: true,
        multivariate: true,
        probabilistic: true,
        backtesting_ready: false,
        anomalies_ready: false,
        fine_tuning_ready: false,
    }
}

const fn advanced_cloud_capabilities() -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates: true,
        future_covariates: true,
        multivariate: true,
        probabilistic: true,
        backtesting_ready: true,
        anomalies_ready: true,
        fine_tuning_ready: true,
    }
}
