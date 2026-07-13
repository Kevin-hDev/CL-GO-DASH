use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use reqwest::Client;
use zeroize::{Zeroize, Zeroizing};

use super::vault;
pub(crate) mod validate {
    include!("api_keys_validate.rs");
}

struct VaultState {
    master_key: Zeroizing<Vec<u8>>,
    keys: HashMap<String, Zeroizing<String>>,
}

static STATE: std::sync::LazyLock<Mutex<Option<VaultState>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

include!("api_keys_registry.rs");

struct ZeroizingMap(HashMap<String, String>);

impl Drop for ZeroizingMap {
    fn drop(&mut self) {
        for val in self.0.values_mut() {
            val.zeroize();
        }
    }
}

fn flush_vault(s: &VaultState) -> Result<(), String> {
    let raw = ZeroizingMap(
        s.keys
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().to_string()))
            .collect(),
    );
    vault::write_vault(&s.master_key, &raw.0)
}

fn migrate_raw_prefix(
    master_key: &Zeroizing<Vec<u8>>,
    map: &mut HashMap<String, String>,
) -> Result<(), String> {
    let to_migrate: Vec<String> = map
        .keys()
        .filter(|k| k.starts_with('_') && !k.starts_with(RAW_PREFIX))
        .cloned()
        .collect();
    if to_migrate.is_empty() {
        return Ok(());
    }
    for old_key in &to_migrate {
        let new_key = format!("{RAW_PREFIX}{old_key}");
        if let Some(val) = map.remove(old_key) {
            map.insert(new_key, val);
        }
    }
    vault::write_vault(master_key, map)?;
    eprintln!(
        "[vault] migrated {} raw keys to namespaced prefix",
        to_migrate.len()
    );
    Ok(())
}

pub fn init() -> Result<(), String> {
    let master_key = vault::load_or_create_master_key()?;
    let mut raw_map = ZeroizingMap(vault::read_vault(&master_key)?);
    let marker = vault::vault_path().with_file_name(".vault-migrated");
    if !marker.exists() {
        let legacy = vault::read_legacy_keychain_keys();
        if !legacy.is_empty() {
            for (id, key) in &legacy {
                raw_map
                    .0
                    .entry(id.clone())
                    .or_insert_with(|| key.to_string());
            }
            eprintln!("[vault] migrated {} keys from keychain", legacy.len());
        }
        vault::write_vault(&master_key, &raw_map.0)?;
        let mut registry = read_registry();
        for id in raw_map.0.keys() {
            if !registry.contains(id) {
                registry.push(id.clone());
            }
        }
        let _ = write_registry(&registry);
        let _ = std::fs::write(&marker, b"ok");
    }
    migrate_raw_prefix(&master_key, &mut raw_map.0)?;
    let keys = raw_map
        .0
        .drain()
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
        .ok_or_else(|| "clé non trouvée".to_string())
}

pub fn set_key(provider_id: &str, key: &str) -> Result<(), String> {
    validate::validate_key_input(provider_id, key)?;
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys
        .insert(provider_id.to_string(), Zeroizing::new(key.to_string()));
    flush_vault(s)?;
    add_to_registry(provider_id)
}

pub fn delete_key(provider_id: &str) -> Result<(), String> {
    validate::validate_provider(provider_id)?;
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys.remove(provider_id);
    flush_vault(s)?;
    remove_from_registry(provider_id)
}

include!("api_keys_raw.rs");

pub async fn test_key(provider_id: &str) -> Result<(), String> {
    if crate::services::llm::catalog::find(provider_id).is_some() {
        let p = crate::services::llm::openai_compat::OpenAiCompatProvider::new(provider_id)
            .map_err(|e| e.to_string())?;
        return p.test_connection().await.map_err(|e| e.to_string());
    }
    let key = get_key(provider_id)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let resp = match provider_id {
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
        "nixtla" => client
            .get("https://api.nixtla.io/models")
            .bearer_auth(&*key),
        other => return Err(format!("Provider inconnu : {other}")),
    }
    .send()
    .await
    .map_err(|e| format!("network: {e}"))?;
    check_status(resp)
}

include!("api_keys_test_raw.rs");
include!("api_keys_mcp.rs");

#[cfg(test)]
#[path = "api_keys_validate_tests.rs"]
mod validate_tests;

#[cfg(test)]
#[path = "api_keys_mcp_tests.rs"]
mod mcp_tests;
