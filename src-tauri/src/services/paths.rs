use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    #[cfg(not(target_os = "windows"))]
    {
        dirs::home_dir()
            .expect("cannot resolve home directory")
            .join(".local/share/cl-go-dash")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .expect("cannot resolve APPDATA directory")
            .join("cl-go-dash")
    }
}
