use std::path::PathBuf;

#[cfg(not(test))]
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".local/share/cl-go-dash")
}

#[cfg(test)]
pub fn data_dir() -> PathBuf {
    use std::sync::OnceLock;

    static TEST_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEST_DATA_DIR
        .get_or_init(|| {
            std::env::temp_dir().join("cl-go-dash-tests").join(format!(
                "{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ))
        })
        .clone()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_data_dir_never_targets_user_data() {
        let actual = super::data_dir();
        let production = dirs::home_dir()
            .expect("cannot resolve home directory")
            .join(".local/share/cl-go-dash");

        assert_ne!(actual, production);
        assert!(actual.starts_with(std::env::temp_dir()));
    }
}
