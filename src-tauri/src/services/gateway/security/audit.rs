use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_LINES: usize = 10_000;

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub channel: String,
    pub account_id: String,
    pub user_hash: String,
    pub action: AuditAction,
    pub security_decision: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    MessageReceived,
    MessageSent,
    Blocked,
    RateLimited,
    ChannelStarted,
    ChannelStopped,
    AuthFailed,
}

pub fn hash_user_id(user_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_id.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

pub fn log_audit(entry: &AuditEntry) {
    let path = audit_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let line = match serde_json::to_string(entry) {
        Ok(l) => l,
        Err(_) => return,
    };
    let file = OpenOptions::new().create(true).append(true).open(&path);
    if let Ok(mut f) = file {
        let _ = writeln!(f, "{}", line);
    }
    let _ = maybe_trim(&path);
}

fn audit_path() -> PathBuf {
    crate::services::paths::data_dir().join("logs/gateway-audit.jsonl")
}

fn maybe_trim(path: &PathBuf) -> Result<(), std::io::Error> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= MAX_LINES {
        return Ok(());
    }
    let keep = &lines[lines.len() - MAX_LINES..];
    let tmp = path.with_extension("jsonl.tmp");
    fs::write(&tmp, keep.join("\n") + "\n")?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let a = hash_user_id("user42");
        let b = hash_user_id("user42");
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn different_users_different_hashes() {
        assert_ne!(hash_user_id("alice"), hash_user_id("bob"));
    }

    #[test]
    fn audit_entry_serializes() {
        let entry = AuditEntry {
            timestamp: "2026-05-10T12:00:00Z".into(),
            channel: "telegram".into(),
            account_id: "bot1".into(),
            user_hash: hash_user_id("user42"),
            action: AuditAction::MessageReceived,
            security_decision: None,
            error: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("message_received"));
        assert!(!json.contains("user42"));
    }
}
