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

fn registry_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("configured-providers.json")
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
        std::fs::create_dir_all(parent).map_err(|_| "erreur de stockage".to_string())?;
    }
    let json = serde_json::to_string_pretty(ids).map_err(|e| format!("json: {e}"))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json).map_err(|_| "erreur écriture registre".to_string())?;
    std::fs::rename(&tmp, &path).map_err(|_| "erreur mise à jour registre".to_string())?;
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
        s.keys.iter().map(|(k, v)| (k.clone(), v.as_str().to_string())).collect(),
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
    eprintln!("[vault] migrated {} raw keys to namespaced prefix", to_migrate.len());
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
                raw_map.0.entry(id.clone()).or_insert_with(|| key.to_string());
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
    let keys = raw_map.0.drain().map(|(k, v)| (k, Zeroizing::new(v))).collect();
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    *state = Some(VaultState { master_key, keys });
    Ok(())
}

pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_ref().ok_or("vault not initialized")?;
    s.keys.get(provider_id).cloned().ok_or_else(|| "clé non trouvée".to_string())
}

pub fn set_key(provider_id: &str, key: &str) -> Result<(), String> {
    validate::validate_key_input(provider_id, key)?;
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys.insert(provider_id.to_string(), Zeroizing::new(key.to_string()));
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

pub fn set_key_raw(key_id: &str, value: &str) -> Result<(), String> {
    if key_id.is_empty() || key_id.len() > 128 {
        return Err("identifiant invalide".to_string());
    }
    if value.is_empty() || value.len() > 256 {
        return Err("valeur invalide".to_string());
    }
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    if !s.keys.contains_key(key_id) && s.keys.len() >= MAX_VAULT_ENTRIES {
        return Err("limite d'entrées vault atteinte".to_string());
    }
    s.keys.insert(key_id.to_string(), Zeroizing::new(value.to_string()));
    flush_vault(s)
}

pub fn delete_key_raw(key_id: &str) -> Result<(), String> {
    if key_id.is_empty() || key_id.len() > 128 {
        return Err("identifiant invalide".to_string());
    }
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys.remove(key_id);
    flush_vault(s)
}

pub fn has_key(provider_id: &str) -> bool {
    let state = STATE.lock().ok();
    state.as_ref().and_then(|s| s.as_ref()).map(|s| s.keys.contains_key(provider_id)).unwrap_or(false)
}

pub fn list_configured() -> Vec<String> {
    read_registry()
}

const MAX_RAW_VALUE_LEN: usize = 8192;
const MAX_VAULT_ENTRIES: usize = 500;

const RAW_PREFIX: &str = "raw:";

pub fn set_raw(key: &str, value: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 64 {
        return Err("clé vault invalide".to_string());
    }
    if value.len() > MAX_RAW_VALUE_LEN {
        return Err("valeur vault trop longue".to_string());
    }
    let prefixed = format!("{RAW_PREFIX}{key}");
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    if !s.keys.contains_key(&prefixed) && s.keys.len() >= MAX_VAULT_ENTRIES {
        return Err("limite d'entrées vault atteinte".to_string());
    }
    s.keys.insert(prefixed, Zeroizing::new(value.to_string()));
    flush_vault(s)
}

pub fn get_raw(key: &str) -> Result<Zeroizing<String>, String> {
    let prefixed = format!("{RAW_PREFIX}{key}");
    let state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_ref().ok_or("vault not initialized")?;
    s.keys.get(&prefixed).cloned().ok_or_else(|| "clé non trouvée".to_string())
}

pub fn delete_raw(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 64 {
        return Err("clé vault invalide".to_string());
    }
    let prefixed = format!("{RAW_PREFIX}{key}");
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys.remove(&prefixed);
    flush_vault(s)
}

pub async fn test_key(provider_id: &str) -> Result<(), String> {
    if crate::services::llm::catalog::find(provider_id).is_some() {
        let p = crate::services::llm::openai_compat::OpenAiCompatProvider::new(provider_id)
            .map_err(|e| e.to_string())?;
        return p.test_connection().await.map_err(|e| e.to_string());
    }
    let key = get_key(provider_id)?;
    let client = Client::builder().timeout(Duration::from_secs(10)).build()
        .map_err(|e| format!("http client: {e}"))?;
    let resp = match provider_id {
        "google" => client.get("https://generativelanguage.googleapis.com/v1beta/models")
            .header("x-goog-api-key", key.as_str()),
        "brave" => client.get("https://api.search.brave.com/res/v1/web/search?q=test&count=1")
            .header("X-Subscription-Token", &*key),
        "exa" => client.post("https://api.exa.ai/search").header("x-api-key", &*key)
            .json(&serde_json::json!({ "query": "test", "numResults": 1 })),
        "firecrawl" => client.get("https://api.firecrawl.dev/v2/team/credit-usage").bearer_auth(&*key),
        "nixtla" => client.get("https://api.nixtla.io/models").bearer_auth(&*key),
        other => return Err(format!("Provider inconnu : {other}")),
    }
    .send().await.map_err(|e| format!("network: {e}"))?;
    check_status(resp)
}

include!("api_keys_test_raw.rs");
include!("api_keys_mcp.rs");
