use crate::services::agent_local::tool_web_fetch_ip::{is_172_private, is_ip_private};
use reqwest::Client;
use std::net::IpAddr;
use std::time::Duration;

const MAX_CHARS: usize = 50_000;
const TIMEOUT: Duration = Duration::from_secs(15);
const MIN_READABLE_LEN: usize = 100;

pub async fn fetch_url(url: &str) -> Result<String, String> {
    let (host, resolved_ip) = validate_url_with_ip(url).await?;
    let html = fetch_html_pinned(url, &host, resolved_ip).await?;
    let content = extract_and_convert(&html, url);
    Ok(truncate(&content, MAX_CHARS))
}

async fn fetch_html_pinned(url: &str, host: &str, ip: IpAddr) -> Result<String, String> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .resolve(host, std::net::SocketAddr::new(ip, 0))
        .build()
        .map_err(|e| format!("Erreur client: {e}"))?;
    let resp = client
        .get(url)
        .header("User-Agent", "CL-GO-DASH/1.0")
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Erreur fetch: {e}"))?;

    if resp.status().is_redirection() {
        return Err("Redirection non autorisée".into());
    }
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

fn extract_and_convert(html: &str, url: &str) -> String {
    let readability_result = extract_readability(html, url);
    let source = if readability_result.len() > MIN_READABLE_LEN {
        &readability_result
    } else {
        html
    };
    convert_to_markdown(source)
}

fn extract_readability(html: &str, url: &str) -> String {
    use dom_smoothie::{Config, Readability};
    let config = Config::default();
    Readability::new(html, Some(url), Some(config))
        .ok()
        .and_then(|mut r| r.parse().ok())
        .map(|a| a.text_content.to_string())
        .unwrap_or_default()
}

fn convert_to_markdown(html: &str) -> String {
    use htmd::HtmlToMarkdown;
    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build();
    converter.convert(html).unwrap_or_default()
}

async fn validate_url_with_ip(url: &str) -> Result<(String, IpAddr), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL doit commencer par http:// ou https://".into());
    }
    let authority = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    let host = if authority.starts_with('[') {
        authority
            .split(']')
            .next()
            .unwrap_or("")
            .trim_start_matches('[')
    } else {
        authority.split(':').next().unwrap_or("")
    };

    if host.is_empty() {
        return Err("URL invalide".into());
    }

    let blocked = host == "localhost"
        || host == "0.0.0.0"
        || host == "::1"
        || host == "[::1]"
        || host.starts_with("0177.")
        || host.starts_with("0x7f")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("127.")
        || is_172_private(host)
        || host.starts_with("169.254.")
        || host.starts_with("fc00:")
        || host.starts_with("fd")
        || host.starts_with("fe80:")
        || host.starts_with("::ffff:127.");

    if blocked {
        return Err("URL privée/locale interdite".into());
    }

    let lookup_target = format!("{host}:80");
    let addrs: Vec<_> = tokio::net::lookup_host(&lookup_target)
        .await
        .map_err(|_| "Résolution DNS échouée".to_string())?
        .collect();

    for addr in &addrs {
        if is_ip_private(&addr.ip()) {
            return Err("URL privée/locale interdite".into());
        }
    }

    let first_ip = addrs.first().ok_or("Résolution DNS vide")?.ip();
    Ok((host.to_string(), first_ip))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) async fn validate_url(url: &str) -> Result<(), String> {
    validate_url_with_ip(url).await.map(|_| ())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}... [tronqué]", &s[..end])
    }
}
