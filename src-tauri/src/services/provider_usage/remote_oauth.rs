use super::types::{ProviderBalance, ProviderWindow, RemoteData, BALANCE_LIMIT, WINDOW_LIMIT};

pub fn parse(connection_id: &str, body: &serde_json::Value) -> Option<RemoteData> {
    match connection_id {
        "codex-oauth" => Some(super::remote_codex::parse(body)),
        "moonshot-oauth" => Some(parse_kimi(body)),
        _ => None,
    }
}

fn parse_kimi(body: &serde_json::Value) -> RemoteData {
    let mut windows = Vec::new();
    if let Some(usage) = body.get("usage").and_then(serde_json::Value::as_object) {
        if let Some(window) = kimi_window(&serde_json::Value::Object(usage.clone()), "weekly") {
            windows.push(window);
        }
    }
    if let Some(limits) = body["limits"].as_array() {
        for item in limits.iter().take(WINDOW_LIMIT) {
            let detail = item
                .get("detail")
                .filter(|value| value.is_object())
                .unwrap_or(item);
            let duration = item
                .pointer("/window/duration")
                .and_then(serde_json::Value::as_u64);
            let unit = item
                .pointer("/window/timeUnit")
                .and_then(serde_json::Value::as_str);
            let label = if duration == Some(300) && unit.is_some_and(|unit| unit.contains("MINUTE"))
            {
                "rolling_five_hours"
            } else {
                "provider_limit"
            };
            if let Some(window) = kimi_window(detail, label) {
                windows.push(window);
            }
        }
    }
    finish(windows, Vec::new(), None)
}

fn kimi_window(value: &serde_json::Value, label: &str) -> Option<ProviderWindow> {
    let limit = finite(value["limit"].as_f64());
    let remaining = finite(value["remaining"].as_f64());
    let used = finite(value["used"].as_f64()).or_else(|| match (limit, remaining) {
        (Some(limit), Some(remaining)) => Some((limit - remaining).max(0.0)),
        _ => None,
    });
    if used.is_none() && limit.is_none() {
        return None;
    }
    Some(ProviderWindow {
        label_code: label.into(),
        group_code: None,
        group_name: None,
        used,
        limit,
        remaining,
        used_percent: match (used, limit) {
            (Some(used), Some(limit)) if limit > 0.0 => {
                Some((used / limit * 100.0).clamp(0.0, 100.0))
            }
            _ => None,
        },
        resets_at: ["reset_at", "resetAt", "reset_time", "resetTime"]
            .iter()
            .find_map(|key| value.get(*key).and_then(parse_reset))
            .or_else(|| reset_after(value)),
    })
}

fn finish(
    mut windows: Vec<ProviderWindow>,
    mut balances: Vec<ProviderBalance>,
    notice_code: Option<String>,
) -> RemoteData {
    windows.truncate(WINDOW_LIMIT);
    balances.truncate(BALANCE_LIMIT);
    RemoteData {
        windows,
        balances,
        notice_code,
        fetched_at: chrono::Utc::now().timestamp(),
        stale: false,
    }
}

fn finite(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite() && *value >= 0.0 && *value <= 1e15)
}

fn timestamp(value: &serde_json::Value) -> Option<i64> {
    value.as_i64().and_then(super::types::valid_reset_timestamp)
}

fn parse_reset(value: &serde_json::Value) -> Option<i64> {
    timestamp(value).or_else(|| {
        let raw = value.as_str().filter(|value| value.len() <= 64)?;
        chrono::DateTime::parse_from_rfc3339(raw)
            .ok()
            .and_then(|date| super::types::valid_reset_timestamp(date.timestamp()))
    })
}

fn reset_after(value: &serde_json::Value) -> Option<i64> {
    let seconds = ["reset_in", "resetIn", "ttl", "window"]
        .iter()
        .find_map(|key| value.get(*key).and_then(parse_seconds))?;
    super::types::valid_reset_timestamp(
        chrono::Utc::now()
            .timestamp()
            .saturating_add(seconds as i64),
    )
}

fn parse_seconds(value: &serde_json::Value) -> Option<u64> {
    let seconds = value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())?;
    (seconds <= 31_536_000).then_some(seconds)
}
