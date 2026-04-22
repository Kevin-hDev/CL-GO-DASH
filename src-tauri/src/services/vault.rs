use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chacha20poly1305::{aead::{Aead, KeyInit}, XChaCha20Poly1305, XNonce};
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "cl-go-dash";
const MASTER_KEY_USER: &str = "master-key";
const VAULT_VERSION: u8 = 1;

const KNOWN_PROVIDERS: &[&str] = &[
    "groq", "google", "mistral", "cerebras", "openrouter",
    "openai", "deepseek", "brave", "exa", "firecrawl", "serpapi", "google_cse",
];

#[derive(Serialize, Deserialize)]
struct VaultFile {
    version: u8,
    nonce: String,
    data: String,
}

pub fn vault_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("secrets.enc")
}

pub fn load_or_create_master_key() -> Result<Zeroizing<Vec<u8>>, String> {
    let entry =
        Entry::new(KEYRING_SERVICE, MASTER_KEY_USER).map_err(|e| format!("keyring entry: {e}"))?;

    if let Ok(b64) = entry.get_password() {
        let bytes = B64.decode(&b64).map_err(|e| format!("master key decode: {e}"))?;
        if bytes.len() == 32 {
            return Ok(Zeroizing::new(bytes));
        }
    }

    let mut key = vec![0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    let b64 = B64.encode(&key);
    entry
        .set_password(&b64)
        .map_err(|e| format!("keyring set master: {e}"))?;
    Ok(Zeroizing::new(key))
}

fn encrypt(master_key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(master_key).map_err(|e| format!("cipher init: {e}"))?;

    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("encrypt: {e}"))?;

    let vault_file = VaultFile {
        version: VAULT_VERSION,
        nonce: B64.encode(nonce_bytes),
        data: B64.encode(&ciphertext),
    };
    serde_json::to_vec_pretty(&vault_file).map_err(|e| format!("json: {e}"))
}

fn decrypt(master_key: &[u8], vault_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let vf: VaultFile =
        serde_json::from_slice(vault_bytes).map_err(|e| format!("vault parse: {e}"))?;
    if vf.version != VAULT_VERSION {
        return Err(format!("unsupported vault version: {}", vf.version));
    }

    let nonce_bytes = B64.decode(&vf.nonce).map_err(|e| format!("nonce decode: {e}"))?;
    let ciphertext = B64.decode(&vf.data).map_err(|e| format!("data decode: {e}"))?;

    let nonce = XNonce::from_slice(&nonce_bytes);
    let cipher =
        XChaCha20Poly1305::new_from_slice(master_key).map_err(|e| format!("cipher init: {e}"))?;

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("decrypt: {e}"))
}

pub fn read_vault(master_key: &[u8]) -> Result<HashMap<String, String>, String> {
    let path = vault_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read vault: {e}"))?;
    let mut plaintext = decrypt(master_key, &bytes)?;
    let result = serde_json::from_slice(&plaintext).map_err(|e| format!("vault json: {e}"));
    plaintext.zeroize();
    result
}

pub fn write_vault(master_key: &[u8], map: &HashMap<String, String>) -> Result<(), String> {
    let path = vault_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let mut plaintext = serde_json::to_vec(map).map_err(|e| format!("json: {e}"))?;
    let encrypted = encrypt(master_key, &plaintext)?;
    plaintext.zeroize();

    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &encrypted).map_err(|e| format!("write: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

pub fn read_legacy_keychain_keys() -> HashMap<String, String> {
    let mut found = HashMap::new();
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, "brave_api_key") {
        if let Ok(key) = entry.get_password() {
            found.insert("brave".to_string(), key);
        }
    }
    for id in KNOWN_PROVIDERS {
        if found.contains_key(*id) {
            continue;
        }
        let Ok(entry) = Entry::new(KEYRING_SERVICE, id) else { continue };
        if let Ok(key) = entry.get_password() {
            found.insert(id.to_string(), key);
        }
    }
    found
}
