use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::RwLock;

static REGISTRY: OnceLock<RwLock<HashMap<String, ModelEntry>>> = OnceLock::new();

const EMBEDDED_JSON: &str = include_str!("../../../resources/litellm-models.json");
const GITHUB_RAW_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

#[derive(Debug, Clone, Deserialize)]
pub struct ModelEntry {
    pub litellm_provider: Option<String>,
    pub max_input_tokens: Option<u64>,
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_function_calling: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelCapabilities {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub max_input_tokens: Option<u64>,
}

fn cache_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".local/share/cl-go-dash/litellm-models.json")
}

fn parse_registry(json: &str) -> HashMap<String, ModelEntry> {
    serde_json::from_str(json).unwrap_or_default()
}

fn get_lock() -> &'static RwLock<HashMap<String, ModelEntry>> {
    REGISTRY.get_or_init(|| {
        let data = std::fs::read_to_string(cache_path())
            .ok()
            .and_then(|s| {
                let map = parse_registry(&s);
                if map.len() > 100 { Some(map) } else { None }
            })
            .unwrap_or_else(|| parse_registry(EMBEDDED_JSON));
        RwLock::new(data)
    })
}

pub async fn init() {
    let _ = get_lock();
    tokio::spawn(async { refresh_from_github().await });
}

async fn refresh_from_github() {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let cached = cache_path();
    let mut req = client.get(GITHUB_RAW_URL);
    if let Ok(meta) = std::fs::metadata(&cached) {
        if let Ok(modified) = meta.modified() {
            let http_date = httpdate::fmt_http_date(modified);
            req = req.header("If-Modified-Since", http_date);
        }
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(_) => return,
    };

    if resp.status() == 304 {
        return;
    }
    if !resp.status().is_success() {
        return;
    }

    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return,
    };

    let map = parse_registry(&body);
    if map.len() < 100 {
        return;
    }

    if let Some(parent) = cached.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cached, &body);

    let mut reg = get_lock().write().await;
    *reg = map;
}

fn map_provider_prefix(provider_id: &str) -> &str {
    match provider_id {
        "google" => "gemini",
        _ => provider_id,
    }
}

pub async fn lookup(provider_id: &str, model_id: &str) -> Option<ModelCapabilities> {
    let reg = get_lock().read().await;
    let prefix = map_provider_prefix(provider_id);

    let key_prefixed = format!("{prefix}/{model_id}");
    let entry = reg.get(&key_prefixed)
        .or_else(|| reg.get(model_id))
        .or_else(|| {
            let stripped = model_id.rsplit_once('/').map(|(_, n)| n).unwrap_or(model_id);
            let key2 = format!("{prefix}/{stripped}");
            reg.get(&key2).or_else(|| reg.get(stripped))
        })?;

    if !is_chat_mode(entry.mode.as_deref()) {
        return None;
    }

    Some(ModelCapabilities {
        supports_tools: entry.supports_function_calling,
        supports_vision: entry.supports_vision,
        supports_thinking: entry.supports_reasoning,
        max_input_tokens: entry.max_input_tokens,
    })
}

fn is_chat_mode(mode: Option<&str>) -> bool {
    matches!(mode, Some("chat") | Some("completion") | None)
}

pub async fn is_chat_model(provider_id: &str, model_id: &str) -> bool {
    let reg = get_lock().read().await;
    let prefix = map_provider_prefix(provider_id);
    let key = format!("{prefix}/{model_id}");
    let entry = reg.get(&key)
        .or_else(|| reg.get(model_id))
        .or_else(|| {
            let stripped = model_id.rsplit_once('/').map(|(_, n)| n).unwrap_or(model_id);
            let key2 = format!("{prefix}/{stripped}");
            reg.get(&key2).or_else(|| reg.get(stripped))
        });
    match entry {
        Some(e) => is_chat_mode(e.mode.as_deref()),
        None => !is_non_chat_name(model_id),
    }
}

fn is_non_chat_name(model_id: &str) -> bool {
    let id = model_id.to_lowercase();
    let non_chat = [
        "whisper", "dall-e", "tts", "embedding", "embed",
        "moderation", "rerank", "lyria", "imagen", "veo",
        "music", "sora", "gpt-image", "stable-diffusion",
    ];
    non_chat.iter().any(|kw| id.contains(kw))
}

pub async fn model_count() -> usize {
    get_lock().read().await.len()
}
