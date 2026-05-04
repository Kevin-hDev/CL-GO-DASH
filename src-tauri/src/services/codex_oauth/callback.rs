use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const BIND_ADDR: &str = "127.0.0.1:1455";
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug)]
pub struct CallbackResult {
    pub code: String,
}

pub async fn wait_for_callback(expected_state: &str) -> Result<CallbackResult, String> {
    let listener = TcpListener::bind(BIND_ADDR)
        .await
        .map_err(|e| format!("bind {BIND_ADDR}: {e}"))?;

    tokio::time::timeout(TIMEOUT, accept_one(&listener, expected_state))
        .await
        .map_err(|_| "aucun callback OAuth reçu en 120s".to_string())?
}

async fn accept_one(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<CallbackResult, String> {
    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("accept: {e}"))?;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("read: {e}"))?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let result = parse_callback(&request, expected_state);

    let (status, body) = match &result {
        Ok(_) => ("200 OK", "Connexion réussie — vous pouvez fermer cet onglet."),
        Err(e) => ("400 Bad Request", e.as_str()),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{body}"
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;

    result
}

fn parse_callback(request: &str, expected_state: &str) -> Result<CallbackResult, String> {
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");
    let query = path.split('?').nth(1).unwrap_or("");

    let mut code = None;
    let mut state = None;
    let mut error = None;

    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("code"), Some(v)) => code = Some(v.to_string()),
            (Some("state"), Some(v)) => state = Some(v.to_string()),
            (Some("error"), Some(v)) => error = Some(v.to_string()),
            _ => {}
        }
    }

    if let Some(err) = error {
        return Err(format!("OAuth error: {err}"));
    }
    let code = code.ok_or("paramètre 'code' manquant dans le callback")?;
    let state = state.ok_or("paramètre 'state' manquant dans le callback")?;

    if state != expected_state {
        return Err("state OAuth invalide (possible CSRF)".to_string());
    }

    Ok(CallbackResult { code })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_callback() {
        let req = "GET /auth/callback?code=abc123&state=mystate HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let result = parse_callback(req, "mystate").unwrap();
        assert_eq!(result.code, "abc123");
    }

    #[test]
    fn rejects_wrong_state() {
        let req = "GET /auth/callback?code=abc&state=wrong HTTP/1.1\r\n\r\n";
        assert!(parse_callback(req, "expected").is_err());
    }

    #[test]
    fn handles_oauth_error() {
        let req = "GET /auth/callback?error=access_denied HTTP/1.1\r\n\r\n";
        let err = parse_callback(req, "any").unwrap_err();
        assert!(err.contains("access_denied"));
    }

    #[test]
    fn rejects_missing_code() {
        let req = "GET /auth/callback?state=mystate HTTP/1.1\r\n\r\n";
        assert!(parse_callback(req, "mystate").is_err());
    }
}
