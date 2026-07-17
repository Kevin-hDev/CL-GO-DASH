use super::{parse_login_hints, ProviderId};
use tokio::io::{AsyncRead, AsyncReadExt};

const MAX_LOGIN_OUTPUT: usize = 16 * 1024;
const MAX_ERROR_MESSAGE: usize = 2 * 1024;

#[derive(Default)]
pub struct LoginOutputSummary {
    pub account_access_required: bool,
}

pub async fn read<R: AsyncRead + Unpin>(
    mut output: R,
    app: tauri::AppHandle,
    provider: ProviderId,
) -> LoginOutputSummary {
    let mut collected = Vec::new();
    let mut chunk = [0u8; 1024];
    let mut last_hints = None;
    while collected.len() < MAX_LOGIN_OUTPUT {
        let read_len = (MAX_LOGIN_OUTPUT - collected.len()).min(chunk.len());
        let Ok(count) = output.read(&mut chunk[..read_len]).await else {
            break;
        };
        if count == 0 {
            break;
        }
        collected.extend_from_slice(&chunk[..count]);
        let raw = String::from_utf8_lossy(&collected);
        let hints = parse_login_hints(&raw);
        if hints != Default::default() && Some(&hints) != last_hints.as_ref() {
            super::login_progress::emit_verification(&app, provider, &hints);
            last_hints = Some(hints);
        }
    }
    LoginOutputSummary {
        account_access_required: provider == ProviderId::Moonshot
            && kimi_account_access_required(&String::from_utf8_lossy(&collected)),
    }
}

fn kimi_account_access_required(raw: &str) -> bool {
    raw.lines().any(|line| {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        if value.get("type").and_then(|item| item.as_str()) != Some("error") {
            return false;
        }
        let Some(message) = value.get("message").and_then(|item| item.as_str()) else {
            return false;
        };
        if message.len() > MAX_ERROR_MESSAGE {
            return false;
        }
        let message = message.to_ascii_lowercase();
        message.contains("no models available")
            || (message.contains("failed to get models")
                && (message.contains("402") || message.contains("payment required")))
    })
}

#[cfg(test)]
mod tests {
    use super::kimi_account_access_required;

    #[test]
    fn detects_the_kimi_account_error_without_matching_unrelated_failures() {
        assert!(kimi_account_access_required(
            r#"{"type":"error","message":"Failed to get models: 402 Payment Required"}"#
        ));
        assert!(kimi_account_access_required(
            r#"{"type":"error","message":"No models available for the selected platform."}"#
        ));
        assert!(!kimi_account_access_required(
            r#"{"type":"error","message":"Login failed: network unavailable"}"#
        ));
    }
}
