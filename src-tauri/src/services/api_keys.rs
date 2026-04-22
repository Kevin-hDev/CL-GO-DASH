use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use reqwest::Client;
use zeroize::Zeroizing;

use super::vault;

struct VaultState {
    master_key: Zeroizing<Vec<u8>>,
    keys: HashMap<String, Zeroizing<String>>,
}

static STATE: std::sync::LazyLock<Mutex<Option<VaultState>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

fn registry_path() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home directory");
    home.join(".local/share/cl-go-dash/configured-providers.json")
}

fn read_registry() -> Vec<String> {
    let path = registry_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn write_registry(ids: &[String]) -> Result<(), String> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(ids).map_err(|e| format!("json: {e}"))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json).map_err(|e| format!("write: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

fn add_to_registry(provider_id: &str) -> Result<(), String> {
    let mut ids = read_registry();
    if !ids.iter().any(|id| id == provider_id) {
        ids.push(provider_id.to_string());
        write_registry(&ids)?;
    }
    Ok(())
}

fn remove_from_registry(provider_id: &str) -> Result<(), String> {
    let mut ids = read_registry();
    let before = ids.len();
    ids.retain(|id| id != provider_id);
    if ids.len() != before {
        write_registry(&ids)?;
    }
    Ok(())
}

fn flush_vault(s: &VaultState) -> Result<(), String> {
    let raw: HashMap<String, String> = s
        .keys
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().to_string()))
        .collect();
    vault::write_vault(&s.master_key, &raw)
}

pub fn init() -> Result<(), String> {
    let master_key = vault::load_or_create_master_key()?;
    let mut raw_map = vault::read_vault(&master_key)?;

    let marker = vault::vault_path().with_file_name(".vault-migrated");
    if !marker.exists() {
        let legacy = vault::read_legacy_keychain_keys();
        if !legacy.is_empty() {
            for (id, key) in &legacy {
                raw_map.entry(id.clone()).or_insert_with(|| key.clone());
            }
            eprintln!("[vault] migrated {} keys from keychain", legacy.len());
        }
        vault::write_vault(&master_key, &raw_map)?;
        let mut registry = read_registry();
        for id in raw_map.keys() {
            if !registry.contains(id) {
                registry.push(id.clone());
            }
        }
        let _ = write_registry(&registry);
        let _ = std::fs::write(&marker, b"ok");
    }

    let keys = raw_map
        .into_iter()
        .map(|(k, v)| (k, Zeroizing::new(v)))
        .collect();
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    *state = Some(VaultState { master_key, keys });
    Ok(())
}

pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_ref().ok_or("vault not initialized")?;
    s.keys
        .get(provider_id)
        .cloned()
        .ok_or_else(|| format!("no key for {provider_id}"))
}

pub fn set_key(provider_id: &str, key: &str) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys
        .insert(provider_id.to_string(), Zeroizing::new(key.to_string()));
    flush_vault(s)?;
    add_to_registry(provider_id)?;
    Ok(())
}

pub fn delete_key(provider_id: &str) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys.remove(provider_id);
    flush_vault(s)?;
    remove_from_registry(provider_id)?;
    Ok(())
}

pub fn has_key(provider_id: &str) -> bool {
    read_registry().iter().any(|id| id == provider_id)
}

pub fn list_configured() -> Vec<String> {
    read_registry()
}

pub async fn test_key(provider_id: &str) -> Result<(), String> {
    let key = get_key(provider_id)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let resp = match provider_id {
        "groq" => client.get("https://api.groq.com/openai/v1/models").bearer_auth(&*key),
        "openai" => client.get("https://api.openai.com/v1/models").bearer_auth(&*key),
        "openrouter" => client.get("https://openrouter.ai/api/v1/models").bearer_auth(&*key),
        "cerebras" => client.get("https://api.cerebras.ai/v1/models").bearer_auth(&*key),
        "mistral" => client.get("https://api.mistral.ai/v1/models").bearer_auth(&*key),
        "deepseek" => client.get("https://api.deepseek.com/v1/models").bearer_auth(&*key),
        "google" => client
            .get("https://generativelanguage.googleapis.com/v1beta/models")
            .header("x-goog-api-key", key.as_str()),
        "brave" => client
            .get("https://api.search.brave.com/res/v1/web/search?q=test&count=1")
            .header("X-Subscription-Token", &*key),
        "exa" => client
            .post("https://api.exa.ai/search")
            .header("x-api-key", &*key)
            .json(&serde_json::json!({ "query": "test", "numResults": 1 })),
        "firecrawl" => client
            .get("https://api.firecrawl.dev/v2/team/credit-usage")
            .bearer_auth(&*key),
        "serpapi" => client
            .get("https://serpapi.com/account")
            .header("Authorization", format!("Bearer {}", key.as_str())),
        "google_cse" => return Ok(()),
        other => return Err(format!("Provider inconnu : {}", other)),
    }
    .send()
    .await
    .map_err(|e| format!("network: {e}"))?;

    match resp.status().as_u16() {
        200..=299 => Ok(()),
        401 | 403 => Err("Clé API invalide ou non autorisée".to_string()),
        429 => Err("Rate limit atteint — clé valide mais quota dépassé".to_string()),
        status => Err(format!("Erreur serveur : HTTP {}", status)),
    }
}
