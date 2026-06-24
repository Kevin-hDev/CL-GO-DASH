#[cfg(target_os = "macos")]
use super::app_update_install_paths::current_macos_app_bundle;
#[cfg(any(target_os = "macos", target_os = "linux", test))]
use super::app_update_install_paths::sh_quote_path;
#[cfg(target_os = "windows")]
use super::app_update_install_paths::{
    batch_escape_text, batch_quote_path, current_windows_install_dir,
};

pub(crate) fn spawn_update_script(path: &std::path::Path) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        spawn_macos_update(path)
    } else if cfg!(target_os = "windows") {
        spawn_windows_update(path)
    } else {
        spawn_linux_update(path)
    }
}

fn update_log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("cl-go-update.log")
}

#[cfg(target_os = "macos")]
fn spawn_macos_update(dmg: &std::path::Path) -> Result<(), String> {
    let dmg_str = sh_quote_path(dmg);
    let target_app = current_macos_app_bundle()?;
    let target_app_str = sh_quote_path(&target_app);
    let log = sh_quote_path(&update_log_path());
    let script = format!(
        r#"#!/bin/bash
exec >> {log} 2>&1
echo "=== update $(date) ==="
sleep 1
while pgrep -x "cl-go-dash" > /dev/null 2>&1; do sleep 0.5; done
VOL=$(hdiutil attach {dmg_str} -nobrowse -noverify 2>&1 | grep -o '/Volumes/.*' | head -1 | sed 's/[[:space:]]*$//')
echo "vol=[$VOL]"
if [ -z "$VOL" ] || [ ! -d "$VOL" ]; then echo "mount failed"; exit 1; fi
if [ -d "$VOL/CL-GO.app" ]; then
  rm -rf {target_app_str}
  cp -Rf "$VOL/CL-GO.app" {target_app_str}
fi
hdiutil detach "$VOL" -quiet 2>/dev/null
rm -f {dmg_str}
open {target_app_str}
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
fn spawn_linux_update(deb: &std::path::Path) -> Result<(), String> {
    run_shell_script(&linux_update_script(deb))
}

#[cfg(any(target_os = "linux", test))]
fn linux_update_script(deb: &std::path::Path) -> String {
    let src = sh_quote_path(deb);
    let log = sh_quote_path(&update_log_path());
    format!(
        r#"#!/bin/bash
exec >> {log} 2>&1
echo "=== update $(date) ==="
sleep 1
while pgrep -x "cl-go-dash" > /dev/null 2>&1; do sleep 0.5; done
if command -v pkexec > /dev/null 2>&1; then
  pkexec apt-get install -y {src}
elif command -v x-terminal-emulator > /dev/null 2>&1; then
  x-terminal-emulator -e bash -lc "sudo apt-get install -y {src}; status=\$?; rm -f {src}; if [ \"\$status\" -eq 0 ]; then cl-go-dash >/dev/null 2>&1 & fi; exit \"\$status\""
  exit 0
else
  sudo apt-get install -y {src}
fi
status=$?
if [ "$status" -ne 0 ]; then echo "install failed"; exit "$status"; fi
rm -f {src}
cl-go-dash >/dev/null 2>&1 &
echo "done"
"#
    )
}

#[cfg(not(target_os = "linux"))]
fn spawn_linux_update(_: &std::path::Path) -> Result<(), String> {
    Err("not Linux".into())
}

#[cfg(target_os = "windows")]
fn spawn_windows_update(installer: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let inst = batch_quote_path(installer)?;
    let install_dir = current_windows_install_dir()?;
    let install_dir_arg = format!("/D={}", install_dir.display());
    let install_dir_arg = batch_escape_text(&install_dir_arg)?;
    let exe = std::env::current_exe().map_err(|e| {
        eprintln!("[update] exe path: {e}");
        "update-install-error".to_string()
    })?;
    let exe_name = exe
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let exe_name = batch_escape_text(&exe_name)?;
    let app = batch_quote_path(&exe)?;
    let log = batch_quote_path(&update_log_path())?;
    let script = format!(
        r#"@echo off
echo === update %date% %time% === >> {log} 2>&1
timeout /t 2 /nobreak >nul
:w
tasklist /fi "imagename eq {exe_name}" 2>nul | find /i "{exe_name}" >nul 2>&1
if not errorlevel 1 (timeout /t 1 /nobreak >nul & goto w)
echo installing >> {log}
start /wait "" {inst} /S "{install_dir_arg}"
del {inst} >nul 2>&1
echo launching >> {log}
start "" {app}
"#
    );
    let path = std::env::temp_dir().join("cl-go-update.bat");
    std::fs::write(&path, &script).map_err(|e| {
        eprintln!("[update] write script: {e}");
        "update-install-error".to_string()
    })?;
    std::process::Command::new("cmd")
        .args(["/C", &path.display().to_string()])
        .creation_flags(0x08000000)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| {
            eprintln!("[update] spawn: {e}");
            "update-install-error".to_string()
        })?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn spawn_windows_update(_: &std::path::Path) -> Result<(), String> {
    Err("not Windows".into())
}

#[cfg(unix)]
fn run_shell_script(content: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join("cl-go-update.sh");
    std::fs::write(&path, content).map_err(|e| {
        eprintln!("[update] write script: {e}");
        "update-install-error".to_string()
    })?;

    std::process::Command::new("bash")
        .arg(&path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| {
            eprintln!("[update] spawn: {e}");
            "update-install-error".to_string()
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn linux_update_script_installs_deb_package() {
        let script = linux_update_script(Path::new("/tmp/CL GO/update.deb"));

        assert!(script.contains("apt-get install -y '/tmp/CL GO/update.deb'"));
        assert!(script.contains("cl-go-dash >/dev/null 2>&1 &"));
        assert!(!script.contains("AppImage"));
    }
}
