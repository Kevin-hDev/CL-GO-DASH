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
        r#"(?i)\b(api[_-]?key|apikey|token|secret|password|authorization|client_secret|access_token|refresh_token)("?\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,}]+)"#,
    )
    .expect("secret value regex")
});

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        (?:
            sk-(?:proj-)?[a-z0-9_-]{8,}
          | bearer[\t ]+[a-z0-9._~+/=-]{8,}
          | gh(?:p|o|u|s|r)_[a-z0-9_-]{8,}
          | github_pat_[a-z0-9_-]{8,}
          | glpat-[a-z0-9_-]{8,}
          | xapp-[a-z0-9-]{8,}
          | xox[a-z]-[a-z0-9-]{8,}
          | [0-9]{5,}:[a-z0-9_-]{20,}
          | (?:AKIA|ASIA)[A-Z0-9]{16}
          | AIza[a-z0-9_-]{35}
          | https://hooks[.]slack[.]com/services/[a-z0-9_-]{8,64}/[a-z0-9_-]{8,64}/[a-z0-9_-]{8,128}
          | [a-z0-9_-]{20,}\.[a-z0-9_-]{5,}\.[a-z0-9_-]{20,}
        )",
    )
    .expect("token regex")
});

static BEARER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)Bearer\s+[^\s,}\"']+"#).expect("bearer regex")
});

static PEM_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)-----BEGIN [A-Z0-9 ]*(PRIVATE KEY|CERTIFICATE)[A-Z0-9 ]*-----.*?-----END [A-Z0-9 ]*(PRIVATE KEY|CERTIFICATE)[A-Z0-9 ]*-----")
        .expect("pem regex")
});

pub fn bash_touches_sensitive_data(command: &str) -> bool {
    // On retire le contenu des heredocs avant l'analyse : un fichier qui
    // mentionne `.env` (un .gitignore par ex.) ne doit pas déclencher l'alerte,
    // seul un vrai chemin `.env` en argument de commande compte.
    let without_heredoc = strip_heredoc_bodies(command);
    let normalized = without_heredoc.replace('\\', "/").to_lowercase();
    SENSITIVE_PATH_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || mentions_protected_app_file(&normalized)
}

pub fn is_sensitive_path(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    SENSITIVE_PATH_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || mentions_protected_app_file(&normalized)
}

/// Retire le corps des heredocs (`<<'EOF' ... EOF` ou `<<EOF ... EOF`) d'une
/// commande shell. Conserve tout le reste (chemins cibles, redirections).
fn strip_heredoc_bodies(command: &str) -> String {
    let mut out = String::with_capacity(command.len());
    let mut delimiter: Option<String> = None;
    for line in command.lines() {
        if delimiter.is_none() {
            if let Some(delim) = heredoc_delimiter(line) {
                // On garde la ligne d'ouverture (contient le chemin/redirect) puis
                // on entre en mode "corps" : les lignes suivantes sont du contenu.
                out.push_str(line);
                out.push('\n');
                delimiter = Some(delim);
            } else {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }
        // En mode corps : on cherche la ligne fermante (le délimiteur seul).
        if line.trim() == delimiter.as_deref().unwrap_or("") {
            delimiter = None;
            out.push_str(line);
            out.push('\n');
        }
        // Sinon on ignore la ligne (contenu du heredoc).
    }
    out
}

/// Extrait le nom du délimiteur d'heredoc depuis une ligne d'ouverture.
/// Gère les formes : `<< EOF`, `<<'EOF'`, `<<"EOF"`, `<<- 'EOF'`, etc.
/// Renvoie `None` si la ligne n'ouvre pas de heredoc.
fn heredoc_delimiter(line: &str) -> Option<String> {
    let l = line.trim();
    let after = l.split("<<").nth(1)?;
    let cleaned: String = after
        .trim()
        .trim_start_matches('-')
        .trim()
        .trim_matches(|c: char| c == '\'' || c == '"')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if cleaned.len() >= 2 {
        Some(cleaned)
    } else {
        None
    }
}

pub fn redact_text(content: &str) -> String {
    let pem_redacted = PEM_BLOCK_RE.replace_all(content, "[REDACTED]");
    let bearer_redacted = BEARER_RE.replace_all(&pem_redacted, "[REDACTED]");
    let tokens_redacted = TOKEN_RE.replace_all(&bearer_redacted, "[REDACTED]");
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
#[path = "sensitive_data_tests.rs"]
mod tests;
