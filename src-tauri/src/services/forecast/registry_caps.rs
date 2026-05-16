use super::ForecastCapabilities;

pub const fn chronos_bolt_caps() -> ForecastCapabilities {
    caps(false, false, false, true, false, false, false)
}

pub const fn chronos_2_caps() -> ForecastCapabilities {
    caps(true, true, true, true, false, false, false)
}

pub const fn univariate_prob_caps() -> ForecastCapabilities {
    caps(false, false, false, true, false, false, false)
}

pub const fn multiseries_prob_caps() -> ForecastCapabilities {
    caps(false, false, true, true, false, false, false)
}

pub const fn toto_caps() -> ForecastCapabilities {
    caps(true, true, true, true, false, false, false)
}

pub const fn cloud_caps() -> ForecastCapabilities {
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
