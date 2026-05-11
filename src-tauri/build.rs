fn main() {
    tauri_build::build();

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=CoreServices");

    load_env("../.env");
}

fn load_env(path: &str) {
    println!("cargo:rerun-if-changed={path}");
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if key.starts_with("CLGO_") {
                println!("cargo:rustc-env={key}={value}");
            }
        }
    }
}
