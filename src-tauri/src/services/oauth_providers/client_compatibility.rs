use super::{OAuthClientState, ProviderId};
use std::io::Read;
use std::path::Path;

const MAX_LAUNCHER_BYTES: u64 = 8 * 1024;

pub(super) fn state(provider: ProviderId, binary: &Path) -> OAuthClientState {
    if provider == ProviderId::Moonshot && is_legacy_kimi(binary) {
        OAuthClientState::Incompatible
    } else {
        OAuthClientState::Ready
    }
}

fn is_legacy_kimi(binary: &Path) -> bool {
    let normalized = binary.to_string_lossy().replace('\\', "/");
    if normalized.contains("/uv/tools/kimi-cli/") {
        return true;
    }
    let Ok(file) = std::fs::File::open(binary) else {
        return false;
    };
    let mut bytes = Vec::with_capacity(MAX_LAUNCHER_BYTES as usize);
    if file
        .take(MAX_LAUNCHER_BYTES)
        .read_to_end(&mut bytes)
        .is_err()
    {
        return false;
    }
    bytes
        .windows(b"kimi_cli.cli".len())
        .any(|window| window == b"kimi_cli.cli")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_python_launcher_is_incompatible() {
        let root = tempfile::tempdir().expect("temporary launcher directory");
        let launcher = root.path().join("kimi");
        std::fs::write(&launcher, b"from kimi_cli.cli import cli\n").expect("legacy launcher");

        assert_eq!(
            state(ProviderId::Moonshot, &launcher),
            OAuthClientState::Incompatible
        );
    }

    #[test]
    fn current_launcher_and_grok_are_ready() {
        let root = tempfile::tempdir().expect("temporary launcher directory");
        let launcher = root.path().join("kimi");
        std::fs::write(&launcher, b"#!/usr/bin/env node\n").expect("current launcher");

        assert_eq!(
            state(ProviderId::Moonshot, &launcher),
            OAuthClientState::Ready
        );
        assert_eq!(state(ProviderId::Xai, &launcher), OAuthClientState::Ready);
    }
}
