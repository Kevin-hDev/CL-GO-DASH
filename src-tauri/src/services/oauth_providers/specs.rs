use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    #[serde(rename = "openai")]
    OpenAi,
    Moonshot,
    Xai,
}

impl ProviderId {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "openai" => Ok(Self::OpenAi),
            "moonshot" => Ok(Self::Moonshot),
            "xai" => Ok(Self::Xai),
            _ => Err("Provider OAuth invalide".to_string()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Moonshot => "moonshot",
            Self::Xai => "xai",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessKind {
    Login,
    Logout,
    Acp,
}

pub struct CommandSpec {
    pub program: &'static str,
    pub args: &'static [&'static str],
}

pub fn command_spec(provider: ProviderId, kind: ProcessKind) -> CommandSpec {
    match (provider, kind) {
        (ProviderId::Moonshot, ProcessKind::Login) => CommandSpec {
            program: "kimi",
            args: &["login"],
        },
        (ProviderId::Moonshot, ProcessKind::Acp) => CommandSpec {
            program: "kimi",
            args: &["acp"],
        },
        (ProviderId::Xai, ProcessKind::Login) => CommandSpec {
            program: "grok",
            args: &["login", "--device-auth"],
        },
        (ProviderId::Xai, ProcessKind::Logout) => CommandSpec {
            program: "grok",
            args: &["logout"],
        },
        (ProviderId::Xai, ProcessKind::Acp) => CommandSpec {
            program: "grok",
            args: &[
                "--no-auto-update",
                "--disallowed-tools",
                "Bash",
                "--deny",
                "Bash(*)",
                "agent",
                "stdio",
            ],
        },
        _ => CommandSpec {
            program: "",
            args: &[],
        },
    }
}

pub fn profile_env_names(provider: ProviderId) -> &'static [&'static str] {
    match provider {
        ProviderId::OpenAi => &[],
        ProviderId::Moonshot => &["KIMI_CODE_HOME", "KIMI_SHARE_DIR"],
        ProviderId::Xai => &["GROK_HOME"],
    }
}

pub fn profile_dir(provider: ProviderId) -> PathBuf {
    crate::services::paths::data_dir()
        .join("oauth-providers")
        .join(provider.as_str())
}

pub fn sanitize_login_output(raw: &str) -> String {
    static URL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"https://[A-Za-z0-9.-]{1,120}(?::[0-9]{1,5})?(?:/[A-Za-z0-9._~:/%+-]{0,180})?")
            .expect("valid login URL regex")
    });
    static CODE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b[A-Z0-9]{4,8}-[A-Z0-9]{4,8}\b").expect("valid device code regex")
    });
    let url = URL.find(raw).map(|item| item.as_str()).unwrap_or("");
    let code = CODE.find(raw).map(|item| item.as_str()).unwrap_or("");
    let result = match (url.is_empty(), code.is_empty()) {
        (false, false) => format!("{url}\n{code}"),
        (false, true) => url.to_string(),
        (true, false) => code.to_string(),
        (true, true) => String::new(),
    };
    result.chars().take(512).collect()
}
