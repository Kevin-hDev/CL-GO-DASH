use super::ForecastModelSpec;

pub const TIMESFM_2_5: ForecastModelSpec = ForecastModelSpec {
    id: "timesfm-2.5-200m",
    provider_id: "google",
    family_id: "timesfm-2-5",
    display_name: "TimesFM 2.5 200M",
    params: "200M",
    size_mb: 925,
    ram_mb: 4200,
    vram_mb: Some(1800),
    cpu_supported: true,
    gpu_supported: true,
    multivariate: false,
    covariates: false,
    horizon_max: 1000,
    frequencies: "Toutes",
    hf_repo: Some("google/timesfm-2.5-200m-pytorch"),
    hf_revision: None,
    github_repo: Some("google-research/timesfm"),
    github_revision: None,
    is_cloud: false,
};
