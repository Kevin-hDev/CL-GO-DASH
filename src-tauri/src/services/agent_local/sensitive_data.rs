use regex::Regex;
use serde_json::{Map, Value};
use std::sync::LazyLock;

pub const PROTECTED_APP_FILES: &[&str] = &[
    "config.json",
    "secrets.enc",
    "agent-settings.json",
    "configured-providers.json",
];

const SENSITIVE_PATH_MARKERS: &[&str] = &[
    ".env",
    ".ssh/",
    "/.ssh",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    ".npmrc",
    ".pypirc",
    ".netrc",
    ".aws/credentials",
    ".config/gcloud",
    ".kube/config",
    "credentials",
    "login.keychain",
    "keychain-db",
];

static SECRET_VALUE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b(api[_-]?key|apikey|token|secret|password|authorization|client_secret|access_token|refresh_token)(\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,}]+)"#,
    )
    .expect("secret value regex")
});

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(sk-|Bearer |ghp_|gho_|glpat-|xox[baprs]-)[A-Za-z0-9_-]{8,}")
        .expect("token regex")
});

static PEM_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)-----BEGIN [A-Z0-9 ]*(PRIVATE KEY|CERTIFICATE)[A-Z0-9 ]*-----.*?-----END [A-Z0-9 ]*(PRIVATE KEY|CERTIFICATE)[A-Z0-9 ]*-----")
        .expect("pem regex")
});

pub fn bash_touches_sensitive_data(command: &str) -> bool {
    let normalized = command.replace('\\', "/").to_lowercase();
    SENSITIVE_PATH_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || mentions_protected_app_file(&normalized)
}

pub fn redact_text(content: &str) -> String {
    let pem_redacted = PEM_BLOCK_RE.replace_all(content, "[REDACTED]");
    let tokens_redacted = TOKEN_RE.replace_all(&pem_redacted, "[REDACTED]");
    SECRET_VALUE_RE
        .replace_all(&tokens_redacted, "$1$2[REDACTED]")
        .into_owned()
}

pub fn redact_json(value: &Value) -> Value {
    redact_json_inner(value, 0)
}

fn mentions_protected_app_file(normalized_command: &str) -> bool {
    let data_dir = crate::services::paths::data_dir()
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    PROTECTED_APP_FILES.iter().any(|file| {
        let full_path = format!("{data_dir}/{file}");
        let app_relative = format!(".local/share/cl-go-dash/{file}");
        let app_folder = format!("cl-go-dash/{file}");
        normalized_command.contains(&full_path)
            || normalized_command.contains(&app_relative)
            || normalized_command.contains(&app_folder)
    })
}

fn redact_json_inner(value: &Value, depth: usize) -> Value {
    if depth > 8 {
        return Value::String("[REDACTED]".to_string());
    }
    match value {
        Value::String(s) => Value::String(redact_text(s)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(64)
                .map(|item| redact_json_inner(item, depth + 1))
                .collect(),
        ),
        Value::Object(map) => {
            let redacted: Map<String, Value> = map
                .iter()
                .take(64)
                .map(|(k, v)| (k.clone(), redact_json_inner(v, depth + 1)))
                .collect();
            Value::Object(redacted)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_env_file() {
        assert!(bash_touches_sensitive_data("cat .env"));
    }

    #[test]
    fn detects_ssh_key() {
        assert!(bash_touches_sensitive_data("head ~/.ssh/id_ed25519"));
    }

    #[test]
    fn detects_app_secret_file() {
        assert!(bash_touches_sensitive_data(
            "cat ~/.local/share/cl-go-dash/secrets.enc"
        ));
    }

    #[test]
    fn ignores_normal_project_search() {
        assert!(!bash_touches_sensitive_data("grep -r token src/"));
    }

    #[test]
    fn redacts_secret_assignments() {
        let text = "API_KEY=abcd PASSWORD: hunter2";
        let redacted = redact_text(text);
        assert!(!redacted.contains("abcd"));
        assert!(!redacted.contains("hunter2"));
    }

    #[test]
    fn redacts_known_token_prefixes() {
        let redacted = redact_text("Authorization: Bearer abcdefghijklmnop");
        assert!(!redacted.contains("abcdefghijklmnop"));
    }

    #[test]
    fn redacts_json_strings() {
        let value = json!({ "command": "echo token=abcdefghi" });
        let redacted = redact_json(&value);
        assert!(!redacted.to_string().contains("abcdefghi"));
    }
}
