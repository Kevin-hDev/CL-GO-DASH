pub fn safe_env() -> Vec<(String, String)> {
    [
        "PATH",
        "HOME",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "USER",
        "SHELL",
        "XDG_DATA_HOME",
        "XDG_CACHE_HOME",
        "XDG_CONFIG_HOME",
        "NODE_PATH",
        "NPM_CONFIG_CACHE",
        "NPM_CONFIG_PREFIX",
        "DENO_DIR",
    ]
    .iter()
    .filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v)))
    .collect()
}
