use serde::Serialize;
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub completed: u64,
    pub total: u64,
}

#[tauri::command]
pub async fn download_app_update(
    app: tauri::AppHandle,
    asset_url: String,
    on_progress: Channel<DownloadProgress>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&asset_url)
        .header("User-Agent", "CL-GO-DASH")
        .send()
        .await
        .map_err(|e| format!("network: {}", e))?;

    if !resp.status().is_success() {
        return Err("download failed".into());
    }

    let total = resp.content_length().unwrap_or(0);
    let ext = if cfg!(target_os = "macos") {
        "dmg"
    } else if cfg!(target_os = "windows") {
        "msi"
    } else {
        "AppImage"
    };
    let tmp = std::env::temp_dir().join(format!("CL-GO-update.{}", ext));

    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("fs: {}", e))?;

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write: {}", e))?;
        downloaded += chunk.len() as u64;
        let _ = on_progress.send(DownloadProgress {
            completed: downloaded,
            total,
        });
    }

    file.flush().await.map_err(|e| format!("flush: {}", e))?;
    drop(file);

    spawn_update_script(&tmp)?;
    app.exit(0);

    Ok(())
}

fn spawn_update_script(path: &std::path::Path) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        spawn_macos_update(path)
    } else if cfg!(target_os = "windows") {
        spawn_windows_update(path)
    } else {
        spawn_linux_update(path)
    }
}

#[cfg(target_os = "macos")]
fn spawn_macos_update(dmg: &std::path::Path) -> Result<(), String> {
    let dmg_str = dmg.display().to_string();
    let script = format!(
        r#"#!/bin/bash
sleep 1
while pgrep -x "CL-GO" > /dev/null 2>&1; do sleep 0.5; done
VOL=$(hdiutil attach "{dmg_str}" -nobrowse -noverify 2>/dev/null | grep "/Volumes/" | sed 's/.*\/Volumes/\/Volumes/')
if [ -z "$VOL" ]; then exit 1; fi
if [ -d "$VOL/CL-GO.app" ]; then
  rm -rf /Applications/CL-GO.app
  cp -Rf "$VOL/CL-GO.app" /Applications/CL-GO.app
fi
hdiutil detach "$VOL" -quiet 2>/dev/null
rm -f "{dmg_str}"
open /Applications/CL-GO.app
"#
    );
    run_shell_script(&script)
}

#[cfg(not(target_os = "macos"))]
fn spawn_macos_update(_: &std::path::Path) -> Result<(), String> {
    Err("not macOS".into())
}

#[cfg(target_os = "linux")]
fn spawn_linux_update(appimage: &std::path::Path) -> Result<(), String> {
    let src = appimage.display().to_string();
    let dest = dirs::home_dir()
        .unwrap_or_default()
        .join(".local/bin/CL-GO.AppImage");
    let dest_str = dest.display().to_string();
    let script = format!(
        r#"#!/bin/bash
sleep 1
while pgrep -f "CL-GO" > /dev/null 2>&1; do sleep 0.5; done
cp -f "{src}" "{dest_str}"
chmod +x "{dest_str}"
rm -f "{src}"
"{dest_str}" &
"#
    );
    run_shell_script(&script)
}

#[cfg(not(target_os = "linux"))]
fn spawn_linux_update(_: &std::path::Path) -> Result<(), String> {
    Err("not Linux".into())
}

#[cfg(target_os = "windows")]
fn spawn_windows_update(msi: &std::path::Path) -> Result<(), String> {
    let msi_str = msi.display().to_string();
    std::process::Command::new("cmd")
        .args(["/C", "start", "/wait", "msiexec", "/i", &msi_str, "/passive"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn: {}", e))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn spawn_windows_update(_: &std::path::Path) -> Result<(), String> {
    Err("not Windows".into())
}

#[cfg(unix)]
fn run_shell_script(content: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join("cl-go-update.sh");
    std::fs::write(&path, content).map_err(|e| format!("script: {}", e))?;

    std::process::Command::new("bash")
        .arg(&path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn: {}", e))?;

    Ok(())
}
