use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".local/share/cl-go-dash")
}
