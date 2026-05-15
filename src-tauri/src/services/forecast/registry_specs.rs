use super::{ForecastCapabilities, ForecastEngineKind as K, ForecastRuntimeSpec};

pub const FORECAST_RUNTIMES: &[ForecastRuntimeSpec] = &[
    rt(
        "chronos-bolt-tiny",
        "chronos-bolt",
        K::LocalChronosBolt,
        simple(),
    ),
    rt(
        "chronos-bolt-mini",
        "chronos-bolt",
        K::LocalChronosBolt,
        simple(),
    ),
    rt(
        "chronos-bolt-small",
        "chronos-bolt",
        K::LocalChronosBolt,
        simple(),
    ),
    rt(
        "chronos-bolt-base",
        "chronos-bolt",
        K::LocalChronosBolt,
        simple(),
    ),
    rt("chronos-2", "chronos-2", K::LocalChronos2, rich_local()),
    rt(
        "timesfm-2.5-200m",
        "timesfm-2-5",
        K::LocalTimesFm,
        multiseries(),
    ),
    rt("timegpt-2-mini", "timegpt-2", K::CloudApi, cloud()),
    rt("timegpt-2-standard", "timegpt-2", K::CloudApi, cloud()),
    rt("timegpt-2-pro", "timegpt-2", K::CloudApi, cloud()),
    rt("timegpt-2.1", "timegpt-2", K::CloudApi, cloud()),
    rt("toto-2.0-4m", "toto-2", K::LocalToto, rich_local()),
    rt("toto-2.0-22m", "toto-2", K::LocalToto, rich_local()),
    rt("toto-2.0-313m", "toto-2", K::LocalToto, rich_local()),
    rt("toto-2.0-1b", "toto-2", K::LocalToto, rich_local()),
    rt("toto-2.0-2.5b", "toto-2", K::LocalToto, rich_local()),
    rt(
        "moirai-2.0-r-small",
        "moirai-2",
        K::LocalMoirai,
        multiseries(),
    ),
    rt(
        "flowstate-r1",
        "flowstate",
        K::LocalFlowState,
        multiseries(),
    ),
    rt(
        "flowstate-r1.1",
        "flowstate",
        K::LocalFlowState,
        multiseries(),
    ),
    rt("tabpfn-ts", "tabpfn-ts", K::LocalTabPfnTs, multiseries()),
    rt("tabpfn-ts-3", "tabpfn-ts", K::LocalTabPfnTs, multiseries()),
    rt("tirex-35m", "tirex", K::LocalTiRex, multiseries()),
    rt("kairos-10m", "kairos", K::LocalKairos, multiseries()),
    rt("kairos-23m", "kairos", K::LocalKairos, multiseries()),
    rt("kairos-50m", "kairos", K::LocalKairos, multiseries()),
    rt("sundial-128m", "sundial", K::LocalSundial, multiseries()),
];

const fn rt(
    model_id: &'static str,
    family_id: &'static str,
    engine_kind: K,
    capabilities: ForecastCapabilities,
) -> ForecastRuntimeSpec {
    ForecastRuntimeSpec {
        model_id,
        family_id,
        engine_kind,
        capabilities,
    }
}

const fn simple() -> ForecastCapabilities {
    caps(false, false, false, true, false, false, false)
}

const fn rich_local() -> ForecastCapabilities {
    caps(true, true, true, true, false, false, false)
}

const fn multiseries() -> ForecastCapabilities {
    caps(false, false, true, true, false, false, false)
}

const fn cloud() -> ForecastCapabilities {
    caps(true, true, true, true, true, true, true)
}

const fn caps(
    past_covariates: bool,
    future_covariates: bool,
    multivariate: bool,
    probabilistic: bool,
    backtesting_ready: bool,
    anomalies_ready: bool,
    fine_tuning_ready: bool,
) -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates,
        future_covariates,
        multivariate,
        probabilistic,
        backtesting_ready,
        anomalies_ready,
        fine_tuning_ready,
    }
}
