use serde::Serialize;

#[path = "catalog_specs.rs"]
mod catalog_specs;

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

pub const FORECAST_PROVIDERS: &[ForecastProviderSpec] = catalog_specs::FORECAST_PROVIDERS;
pub const FORECAST_MODELS: &[ForecastModelSpec] = catalog_specs::FORECAST_MODELS;
