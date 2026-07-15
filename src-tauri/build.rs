fn main() {
    prepare_cef_bundle_placeholders();
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-lib=framework=CoreServices");
    }

    load_env("../.env");
}

fn prepare_cef_bundle_placeholders() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    if target_os.as_deref() == Ok("windows") {
        std::fs::create_dir_all("target/cef-runtime/windows")
            .expect("cannot prepare Windows CEF bundle directory");
        return;
    }
    if target_os.as_deref() != Ok("macos") {
        return;
    }
    let root = std::path::Path::new("target/cef-runtime/macos");
    let framework = root.join("Chromium Embedded Framework.framework");
    let helpers = root.join("helpers");
    if let Err(error) = std::fs::create_dir_all(framework) {
        panic!("cannot prepare CEF bundle directory: {error}");
    }
    if let Err(error) = std::fs::create_dir_all(helpers) {
        panic!("cannot prepare CEF helper directory: {error}");
    }
    let license = root.join("LICENSE.txt");
    if !license.exists() {
        std::fs::File::create(license).expect("cannot prepare CEF license placeholder");
    }
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
