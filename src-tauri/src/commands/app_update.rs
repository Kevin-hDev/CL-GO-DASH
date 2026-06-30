use serde::Serialize;

use super::app_update_assets::{asset_extension, current_platform};
use super::app_update_notes::{
    parse_app_release_notes_json, AppReleaseNotesByLocale, MAX_RELEASE_NOTES_BYTES,
};

const GITHUB_REPO: &str = "Kevin-hDev/CL-GO-DASH";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub version: String,
    pub asset_url: String,
    pub title: Option<String>,
    pub published_at: Option<String>,
    pub notes_by_locale: Option<AppReleaseNotesByLocale>,
}

#[tauri::command]
pub async fn check_app_update() -> Result<Option<AppUpdateInfo>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "CL-GO-DASH")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| {
            eprintln!("[update] check network: {e}");
            "update-check-error".to_string()
        })?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        eprintln!("[update] parse json: {e}");
        "update-check-error".to_string()
    })?;

    let mut update = match app_update_from_release(&json, env!("CARGO_PKG_VERSION")) {
        Some(update) => update,
        None => return Ok(None),
    };
    update.notes_by_locale = fetch_release_notes(&client, &update.version).await;
    Ok(Some(update))
}

fn app_update_from_release(json: &serde_json::Value, current: &str) -> Option<AppUpdateInfo> {
    let tag = json["tag_name"].as_str()?.trim_start_matches('v');
    if !is_safe_version(tag) || !version_gt(tag, current) {
        return None;
    }

    let asset_url = find_platform_asset(json)?;
    Some(AppUpdateInfo {
        version: tag.to_string(),
        asset_url,
        title: optional_string(json["name"].as_str()),
        published_at: optional_string(json["published_at"].as_str()),
        notes_by_locale: None,
    })
}

async fn fetch_release_notes(
    client: &reqwest::Client,
    version: &str,
) -> Option<AppReleaseNotesByLocale> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/v{}/app-release-notes.json",
        GITHUB_REPO, version
    );
    let resp = client
        .get(url)
        .header("User-Agent", "CL-GO-DASH")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }
    if let Some(length) = resp.content_length() {
        if length > MAX_RELEASE_NOTES_BYTES as u64 {
            return None;
        }
    }

    let bytes = resp.bytes().await.ok()?;
    parse_app_release_notes_json(&bytes, version)
}

fn optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn find_platform_asset(json: &serde_json::Value) -> Option<String> {
    find_asset_by_extension(json, asset_extension(current_platform()))
}

fn find_asset_by_extension(json: &serde_json::Value, ext: &str) -> Option<String> {
    let assets = json["assets"].as_array()?;
    assets.iter().find_map(|a| {
        let name = a["name"].as_str().unwrap_or_default();
        if name.ends_with(ext) {
            a["browser_download_url"].as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

pub(crate) fn version_gt(remote: &str, local: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let r = parse(remote);
    let l = parse(local);
    for i in 0..r.len().max(l.len()) {
        let rv = r.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        if rv > lv {
            return true;
        }
        if rv < lv {
            return false;
        }
    }
    false
}

fn is_safe_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && !value.starts_with('.')
        && !value.ends_with('.')
        && value
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
#[path = "app_update_tests.rs"]
mod tests;
