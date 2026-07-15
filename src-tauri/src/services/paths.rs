use std::path::PathBuf;

#[cfg(all(not(test), not(feature = "cef-test-profile")))]
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".local/share/cl-go-dash")
}

#[cfg(all(not(test), feature = "cef-test-profile"))]
pub fn data_dir() -> PathBuf {
    use std::sync::OnceLock;

    static TEST_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEST_DATA_DIR
        .get_or_init(|| resolve_cef_test_data_dir().expect("invalid CEF test data directory"))
        .clone()
}

#[cfg(feature = "cef-test-profile")]
fn resolve_cef_test_data_dir() -> Result<PathBuf, ()> {
    let raw = std::env::var_os("CL_GO_CEF_TEST_DATA_DIR").ok_or(())?;
    let requested = PathBuf::from(raw);
    let temp = std::env::temp_dir().canonicalize().map_err(|_| ())?;
    if !clean_cef_test_path(&requested) {
        return Err(());
    }
    let resolved = requested.canonicalize().map_err(|_| ())?;
    allowed_cef_test_path(&resolved, &temp)
        .then_some(resolved)
        .ok_or(())
}

#[cfg(any(test, feature = "cef-test-profile"))]
fn allowed_cef_test_path(path: &std::path::Path, temp: &std::path::Path) -> bool {
    clean_cef_test_path(path) && path.starts_with(temp)
}

#[cfg(any(test, feature = "cef-test-profile"))]
fn clean_cef_test_path(path: &std::path::Path) -> bool {
    use std::path::Component;

    path.is_absolute()
        && path.to_string_lossy().len() <= 4096
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}

#[cfg(test)]
pub fn data_dir() -> PathBuf {
    use std::sync::OnceLock;

    static TEST_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEST_DATA_DIR
        .get_or_init(|| {
            let path = std::env::temp_dir().join("cl-go-dash-tests").join(format!(
                "{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&path).expect("create isolated test data directory");
            path.canonicalize()
                .expect("canonicalize isolated test data directory")
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_data_dir_never_targets_user_data() {
        let actual = super::data_dir();
        let production = dirs::home_dir()
            .expect("cannot resolve home directory")
            .join(".local/share/cl-go-dash");

        assert_ne!(actual, production);
        let temp = std::env::temp_dir()
            .canonicalize()
            .expect("canonicalize system temp directory");
        assert!(actual.starts_with(temp));
    }

    #[test]
    fn cef_test_profile_accepts_only_clean_absolute_temp_paths() {
        let temp = Path::new("/private/tmp");

        assert!(super::allowed_cef_test_path(
            Path::new("/private/tmp/cl-go-cef-test"),
            temp,
        ));
        assert!(!super::allowed_cef_test_path(
            Path::new("/private/tmp/../Users/kevinh"),
            temp,
        ));
        assert!(!super::allowed_cef_test_path(
            Path::new("/Users/kevinh/.local/share/cl-go-dash"),
            temp,
        ));
    }
}
