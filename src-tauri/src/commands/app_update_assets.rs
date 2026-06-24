#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdatePlatform {
    Macos,
    Windows,
    Linux,
}

pub(crate) fn current_platform() -> UpdatePlatform {
    if cfg!(target_os = "macos") {
        UpdatePlatform::Macos
    } else if cfg!(target_os = "windows") {
        UpdatePlatform::Windows
    } else {
        UpdatePlatform::Linux
    }
}

pub(crate) fn asset_extension(platform: UpdatePlatform) -> &'static str {
    match platform {
        UpdatePlatform::Macos => ".dmg",
        UpdatePlatform::Windows => "-setup.exe",
        UpdatePlatform::Linux => ".deb",
    }
}

pub(crate) fn temp_extension(platform: UpdatePlatform) -> &'static str {
    match platform {
        UpdatePlatform::Macos => "dmg",
        UpdatePlatform::Windows => "exe",
        UpdatePlatform::Linux => "deb",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_uses_deb_asset() {
        assert_eq!(asset_extension(UpdatePlatform::Linux), ".deb");
        assert_eq!(temp_extension(UpdatePlatform::Linux), "deb");
    }

    #[test]
    fn keeps_existing_macos_and_windows_assets() {
        assert_eq!(asset_extension(UpdatePlatform::Macos), ".dmg");
        assert_eq!(asset_extension(UpdatePlatform::Windows), "-setup.exe");
    }
}
