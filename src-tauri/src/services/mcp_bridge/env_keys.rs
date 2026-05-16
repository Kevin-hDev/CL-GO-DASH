const FORBIDDEN_ENV_KEYS: &[&str] = &[
    "PATH",
    "HOME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "USER",
    "SHELL",
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "DYLD_INSERT_LIBRARIES",
    "NODE_OPTIONS",
    "NODE_PATH",
    "DENO_DIR",
    "NPM_CONFIG_CACHE",
    "NPM_CONFIG_PREFIX",
    "XDG_DATA_HOME",
    "XDG_CACHE_HOME",
    "XDG_CONFIG_HOME",
];

pub fn validated_env_keys(keys: Option<&[String]>) -> Vec<String> {
    keys.unwrap_or_default()
        .iter()
        .take(10)
        .filter(|k| is_valid_env_key(k))
        .cloned()
        .collect()
}

fn is_valid_env_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !FORBIDDEN_ENV_KEYS.contains(&key)
}
