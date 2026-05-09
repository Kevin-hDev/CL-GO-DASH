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
        .map_err(|e| { eprintln!("[update] network: {e}"); "update-download-error".to_string() })?;

    if !resp.status().is_success() {
        return Err("download failed".into());
    }

    let total = resp.content_length().unwrap_or(0);
    let ext = if cfg!(target_os = "macos") {
        "dmg"
    } else if cfg!(target_os = "windows") {
        "exe"
    } else {
        "AppImage"
    };
    let tmp = std::env::temp_dir().join(format!("CL-GO-update.{}", ext));

    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| { eprintln!("[update] create file: {e}"); "update-write-error".to_string() })?;

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| { eprintln!("[update] stream: {e}"); "update-download-error".to_string() })?;
        file.write_all(&chunk)
            .await
            .map_err(|e| { eprintln!("[update] write: {e}"); "update-write-error".to_string() })?;
        downloaded += chunk.len() as u64;
        let _ = on_progress.send(DownloadProgress {
            completed: downloaded,
            total,
        });
    }

    file.flush().await.map_err(|e| { eprintln!("[update] flush: {e}"); "update-write-error".to_string() })?;
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

fn update_log_path() -> String {
    std::env::temp_dir().join("cl-go-update.log").display().to_string()
}

#[cfg(target_os = "macos")]
fn spawn_macos_update(dmg: &std::path::Path) -> Result<(), String> {
    let dmg_str = dmg.display().to_string();
    let log = update_log_path();
    let script = format!(
        r#"#!/bin/bash
exec >> "{log}" 2>&1
echo "=== update $(date) ==="
sleep 1
while pgrep -x "cl-go-dash" > /dev/null 2>&1; do sleep 0.5; done
VOL=$(hdiutil attach "{dmg_str}" -nobrowse -noverify 2>&1 | grep -o '/Volumes/.*' | head -1 | sed 's/[[:space:]]*$//')
echo "vol=[$VOL]"
if [ -z "$VOL" ] || [ ! -d "$VOL" ]; then echo "mount failed"; exit 1; fi
if [ -d "$VOL/CL-GO.app" ]; then
  rm -rf /Applications/CL-GO.app
  cp -Rf "$VOL/CL-GO.app" /Applications/CL-GO.app
fi
hdiutil detach "$VOL" -quiet 2>/dev/null
rm -f "{dmg_str}"
open /Applications/CL-GO.app
echo "done"
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
    let log = update_log_path();
    let script = format!(
        r#"#!/bin/bash
exec >> "{log}" 2>&1
echo "=== update $(date) ==="
sleep 1
while pgrep -x "cl-go-dash" > /dev/null 2>&1; do sleep 0.5; done
cp -f "{src}" "{dest_str}"
chmod +x "{dest_str}"
rm -f "{src}"
"{dest_str}" &
echo "done"
"#
    );
    run_shell_script(&script)
}

#[cfg(not(target_os = "linux"))]
fn spawn_linux_update(_: &std::path::Path) -> Result<(), String> {
    Err("not Linux".into())
}

#[cfg(target_os = "windows")]
fn spawn_windows_update(installer: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let inst = installer.display().to_string();
    let exe = std::env::current_exe().map_err(|e| { eprintln!("[update] exe path: {e}"); "update-install-error".to_string() })?;
    let exe_name = exe.file_name().unwrap_or_default().to_string_lossy().to_string();
    let app = exe.display().to_string();
    let log = update_log_path();
    let script = format!(
        r#"@echo off
echo === update %date% %time% === >> "{log}" 2>&1
timeout /t 2 /nobreak >nul
:w
tasklist /fi "imagename eq {exe_name}" 2>nul | find /i "{exe_name}" >nul 2>&1
if not errorlevel 1 (timeout /t 1 /nobreak >nul & goto w)
echo installing >> "{log}"
start /wait "" "{inst}" /S
del "{inst}" >nul 2>&1
echo launching >> "{log}"
start "" "{app}"
"#
    );
    let path = std::env::temp_dir().join("cl-go-update.bat");
    std::fs::write(&path, &script).map_err(|e| { eprintln!("[update] write script: {e}"); "update-install-error".to_string() })?;
    std::process::Command::new("cmd")
        .args(["/C", &path.display().to_string()])
        .creation_flags(0x08000000)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| { eprintln!("[update] spawn: {e}"); "update-install-error".to_string() })?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn spawn_windows_update(_: &std::path::Path) -> Result<(), String> {
    Err("not Windows".into())
}

#[cfg(unix)]
fn run_shell_script(content: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join("cl-go-update.sh");
    std::fs::write(&path, content).map_err(|e| { eprintln!("[update] write script: {e}"); "update-install-error".to_string() })?;

    std::process::Command::new("bash")
        .arg(&path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| { eprintln!("[update] spawn: {e}"); "update-install-error".to_string() })?;

    Ok(())
}
