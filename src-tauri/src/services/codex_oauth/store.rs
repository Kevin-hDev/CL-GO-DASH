use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::services::api_keys;

const VAULT_KEY: &str = "_codex_oauth";

#[derive(Serialize, Deserialize)]
struct Stored {
    access: String,
    refresh: String,
    expires_at: i64,
    account_id: String,
}

pub struct CodexTokens {
    pub access: Zeroizing<String>,
    pub refresh: Zeroizing<String>,
    pub expires_at: i64,
    pub account_id: String,
}

impl CodexTokens {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at - 30
    }
}

impl Drop for CodexTokens {
    fn drop(&mut self) {
        self.account_id.zeroize();
    }
}

pub fn save(tokens: &CodexTokens) -> Result<(), String> {
    let raw = Stored {
        access: tokens.access.to_string(),
        refresh: tokens.refresh.to_string(),
        expires_at: tokens.expires_at,
        account_id: tokens.account_id.clone(),
    };
    let mut json = serde_json::to_string(&raw).map_err(|e| format!("json: {e}"))?;
    let result = api_keys::set_raw(VAULT_KEY, &json);
    json.zeroize();
    result
}

pub fn load() -> Result<Option<CodexTokens>, String> {
    match api_keys::get_raw(VAULT_KEY) {
        Ok(json) => {
            let raw: Stored = serde_json::from_str(&json)
                .map_err(|e| format!("parse codex tokens: {e}"))?;
            Ok(Some(CodexTokens {
                access: Zeroizing::new(raw.access),
                refresh: Zeroizing::new(raw.refresh),
                expires_at: raw.expires_at,
                account_id: raw.account_id,
            }))
        }
        Err(_) => Ok(None),
    }
}

pub fn clear() -> Result<(), String> {
    api_keys::delete_raw(VAULT_KEY)
}

pub fn is_logged_in() -> bool {
    load().ok().flatten().is_some()
}
