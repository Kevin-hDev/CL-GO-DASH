#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

pub fn detect() -> GpuVendor {
    #[cfg(target_os = "linux")]
    {
        return detect_linux();
    }
    #[cfg(target_os = "windows")]
    {
        return detect_windows();
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    GpuVendor::Unknown
}

#[cfg(target_os = "linux")]
fn detect_linux() -> GpuVendor {
    let drm = match std::fs::read_dir("/sys/class/drm") {
        Ok(d) => d,
        Err(_) => return GpuVendor::Unknown,
    };
    let mut best = GpuVendor::Unknown;
    for entry in drm.flatten() {
        let vendor_path = entry.path().join("device/vendor");
        if let Ok(raw) = std::fs::read_to_string(&vendor_path) {
            match raw.trim().to_lowercase().as_str() {
                "0x1002" => return GpuVendor::Amd,
                "0x10de" => return GpuVendor::Nvidia,
                "0x8086" => best = GpuVendor::Intel,
                _ => {}
            }
        }
    }
    best
}

#[cfg(target_os = "windows")]
fn detect_windows() -> GpuVendor {
    use std::process::Command;

    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-Command",
        "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name",
    ]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(_) => return GpuVendor::Unknown,
    };

    let name = String::from_utf8_lossy(&output.stdout).to_lowercase();
    if name.contains("nvidia") || name.contains("geforce") || name.contains("quadro") {
        GpuVendor::Nvidia
    } else if name.contains("amd") || name.contains("radeon") {
        GpuVendor::Amd
    } else if name.contains("intel") {
        GpuVendor::Intel
    } else {
        GpuVendor::Unknown
    }
}
