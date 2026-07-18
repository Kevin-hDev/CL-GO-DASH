use super::model_registry::{get_lock, ModelEntry};

#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub cache_read_input_token_cost: Option<f64>,
}

pub async fn lookup(provider_id: &str, model_id: &str) -> Option<ModelPricing> {
    let registry = get_lock().read().await;
    let entry = find_entry(&registry, provider_id, model_id)?;
    Some(ModelPricing {
        input_cost_per_token: entry.input_cost_per_token,
        output_cost_per_token: entry.output_cost_per_token,
        cache_read_input_token_cost: entry.cache_read_input_token_cost,
    })
}

fn find_entry<'a>(
    registry: &'a std::collections::HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a ModelEntry> {
    let prefix = if provider_id == "google" {
        "gemini"
    } else {
        provider_id
    };
    let prefixed = format!("{prefix}/{model_id}");
    registry
        .get(&prefixed)
        .or_else(|| registry.get(model_id))
        .or_else(|| {
            let stripped = model_id
                .rsplit_once('/')
                .map(|(_, name)| name)
                .unwrap_or(model_id);
            registry
                .get(&format!("{prefix}/{stripped}"))
                .or_else(|| registry.get(stripped))
        })
}
