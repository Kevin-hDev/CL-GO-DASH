use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ForecastProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    pub base_url: &'static str,
    pub signup_url: &'static str,
    pub free_tier_label: &'static str,
    pub short_description: &'static str,
    pub short_description_en: &'static str,
    pub requires_api_key: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForecastModelSpec {
    pub id: &'static str,
    pub provider_id: &'static str,
    pub display_name: &'static str,
    pub params: &'static str,
    pub size_mb: u32,
    pub ram_mb: u32,
    pub vram_mb: Option<u32>,
    pub cpu_supported: bool,
    pub gpu_supported: bool,
    pub multivariate: bool,
    pub covariates: bool,
    pub horizon_max: u32,
    pub frequencies: &'static str,
    pub hf_repo: Option<&'static str>,
    pub is_cloud: bool,
}

pub fn find_provider(id: &str) -> Option<&'static ForecastProviderSpec> {
    FORECAST_PROVIDERS.iter().find(|p| p.id == id)
}

pub fn find_model(id: &str) -> Option<&'static ForecastModelSpec> {
    FORECAST_MODELS.iter().find(|m| m.id == id)
}

pub fn models_for_provider(provider_id: &str) -> Vec<&'static ForecastModelSpec> {
    FORECAST_MODELS
        .iter()
        .filter(|m| m.provider_id == provider_id)
        .collect()
}

pub const FORECAST_PROVIDERS: &[ForecastProviderSpec] = &[
    ForecastProviderSpec {
        id: "chronos",
        display_name: "Chronos-Bolt (Amazon)",
        category: "forecast",
        base_url: "http://localhost:12000",
        signup_url: "https://huggingface.co/amazon/chronos-bolt-small",
        free_tier_label: "Local",
        short_description: "Modèles locaux de séries temporelles — Apache 2.0.",
        short_description_en: "Local time series models — Apache 2.0.",
        requires_api_key: false,
    },
    ForecastProviderSpec {
        id: "nixtla",
        display_name: "Nixtla (TimeGPT-2)",
        category: "forecast",
        base_url: "https://api.nixtla.io",
        signup_url: "https://dashboard.nixtla.io",
        free_tier_label: "$1000 credits",
        short_description: "TimeGPT-2 cloud — $1000 crédits offerts.",
        short_description_en: "TimeGPT-2 cloud — $1000 free credits.",
        requires_api_key: true,
    },
];

pub const FORECAST_MODELS: &[ForecastModelSpec] = &[
    ForecastModelSpec {
        id: "chronos-bolt-tiny",
        provider_id: "chronos",
        display_name: "Chronos-Bolt Tiny",
        params: "9M",
        size_mb: 35,
        ram_mb: 150,
        vram_mb: Some(60),
        cpu_supported: true,
        gpu_supported: true,
        multivariate: false,
        covariates: false,
        horizon_max: 1000,
        frequencies: "10S à Y",
        hf_repo: Some("amazon/chronos-bolt-tiny"),
        is_cloud: false,
    },
    ForecastModelSpec {
        id: "chronos-bolt-mini",
        provider_id: "chronos",
        display_name: "Chronos-Bolt Mini",
        params: "21M",
        size_mb: 85,
        ram_mb: 350,
        vram_mb: Some(120),
        cpu_supported: true,
        gpu_supported: true,
        multivariate: false,
        covariates: false,
        horizon_max: 1000,
        frequencies: "10S à Y",
        hf_repo: Some("amazon/chronos-bolt-mini"),
        is_cloud: false,
    },
    ForecastModelSpec {
        id: "chronos-bolt-small",
        provider_id: "chronos",
        display_name: "Chronos-Bolt Small",
        params: "48M",
        size_mb: 191,
        ram_mb: 750,
        vram_mb: Some(280),
        cpu_supported: true,
        gpu_supported: true,
        multivariate: false,
        covariates: false,
        horizon_max: 1000,
        frequencies: "10S à Y",
        hf_repo: Some("amazon/chronos-bolt-small"),
        is_cloud: false,
    },
    ForecastModelSpec {
        id: "chronos-bolt-base",
        provider_id: "chronos",
        display_name: "Chronos-Bolt Base",
        params: "205M",
        size_mb: 821,
        ram_mb: 3200,
        vram_mb: Some(1200),
        cpu_supported: true,
        gpu_supported: true,
        multivariate: false,
        covariates: false,
        horizon_max: 1000,
        frequencies: "10S à Y",
        hf_repo: Some("amazon/chronos-bolt-base"),
        is_cloud: false,
    },
    ForecastModelSpec {
        id: "timegpt-2-mini",
        provider_id: "nixtla",
        display_name: "TimeGPT-2 Mini",
        params: "—",
        size_mb: 0,
        ram_mb: 0,
        vram_mb: None,
        cpu_supported: false,
        gpu_supported: false,
        multivariate: true,
        covariates: true,
        horizon_max: 5000,
        frequencies: "T à Y",
        hf_repo: None,
        is_cloud: true,
    },
    ForecastModelSpec {
        id: "timegpt-2-standard",
        provider_id: "nixtla",
        display_name: "TimeGPT-2 Standard",
        params: "—",
        size_mb: 0,
        ram_mb: 0,
        vram_mb: None,
        cpu_supported: false,
        gpu_supported: false,
        multivariate: true,
        covariates: true,
        horizon_max: 5000,
        frequencies: "T à Y",
        hf_repo: None,
        is_cloud: true,
    },
    ForecastModelSpec {
        id: "timegpt-2-pro",
        provider_id: "nixtla",
        display_name: "TimeGPT-2 Pro",
        params: "—",
        size_mb: 0,
        ram_mb: 0,
        vram_mb: None,
        cpu_supported: false,
        gpu_supported: false,
        multivariate: true,
        covariates: true,
        horizon_max: 5000,
        frequencies: "T à Y",
        hf_repo: None,
        is_cloud: true,
    },
];
