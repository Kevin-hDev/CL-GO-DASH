use crate::services::agent_local::tool_web_fetch_ip::{is_172_private, is_ip_private};
use reqwest::Client;
use std::net::IpAddr;
use std::time::Duration;

const MAX_CHARS: usize = 50_000;
const TIMEOUT: Duration = Duration::from_secs(15);
const JINA_PREFIX: &str = "https://r.jina.ai/";
const MIN_CONTENT_LEN: usize = 100;

pub async fn fetch_url(url: &str) -> Result<String, String> {
    validate_url(url).await?;
    let html = fetch_html(url).await?;
    let content = extract_and_convert(&html, url);

    if content.len() < MIN_CONTENT_LEN {
        return fetch_via_jina(url).await;
    }
    Ok(truncate(&content, MAX_CHARS))
}

async fn fetch_html(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
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
    let source = if readability_result.len() > MIN_CONTENT_LEN {
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

async fn fetch_via_jina(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Erreur client: {e}"))?;
    let jina_url = format!("{JINA_PREFIX}{url}");
    let resp = client
        .get(&jina_url)
        .header("Accept", "text/markdown")
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Jina fallback: {e}"))?;

    let text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(truncate(&text, MAX_CHARS))
}

pub(crate) async fn validate_url(url: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL doit commencer par http:// ou https://".into());
    }
    let authority = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    // Sépare host et port optionnel, en gérant [::1]:port (IPv6)
    let host = if authority.starts_with('[') {
        authority.split(']').next().unwrap_or("").trim_start_matches('[')
    } else {
        authority.split(':').next().unwrap_or("")
    };

    if host.is_empty() {
        return Err("URL invalide".into());
    }

    // Blocklist statique : patterns obfusqués et plages manquantes
    let blocked = host == "localhost"
        || host == "0.0.0.0"
        || host == "::1"
        || host == "[::1]"
        // Formats octaux/hex de loopback (0177.*, 0x7f.*)
        || host.starts_with("0177.")
        || host.starts_with("0x7f")
        // Plages RFC1918
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("127.")
        || is_172_private(host)
        // Link-local
        || host.starts_with("169.254.")
        // IPv6 ULA / link-local / loopback mappé IPv4
        || host.starts_with("fc00:")
        || host.starts_with("fd")
        || host.starts_with("fe80:")
        || host.starts_with("::ffff:127.");

    if blocked {
        return Err("URL privée/locale interdite".into());
    }

    // Résolution DNS + vérification de l'IP résolue
    let lookup_target = format!("{}:80", host);
    let addrs = tokio::net::lookup_host(&lookup_target)
        .await
        .map_err(|_| "Résolution DNS échouée".to_string())?;

    for addr in addrs {
        let ip: IpAddr = addr.ip();
        if is_ip_private(&ip) {
            return Err("URL privée/locale interdite".into());
        }
    }

    Ok(())
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

