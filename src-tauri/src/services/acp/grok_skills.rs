use std::path::Path;
use toml_edit::{value, Array, DocumentMut, Item, Table};

const MAX_CONFIG_BYTES: u64 = 256 * 1024;
const MAX_SKILLS_PATH_CHARS: usize = 1_024;

pub fn configure(config: &Path, skills: &Path) -> Result<(), String> {
    if !skills.is_absolute() {
        return Err("Configuration Grok indisponible".to_string());
    }
    let skills = skills
        .to_str()
        .filter(|path| !path.is_empty() && path.chars().count() <= MAX_SKILLS_PATH_CHARS)
        .ok_or_else(|| "Configuration Grok indisponible".to_string())?;
    let current = match std::fs::metadata(config) {
        Ok(metadata) if metadata.len() <= MAX_CONFIG_BYTES => std::fs::read_to_string(config)
            .map_err(|_| "Configuration Grok indisponible".to_string())?,
        Ok(_) => return Err("Configuration Grok indisponible".to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(_) => return Err("Configuration Grok indisponible".to_string()),
    };
    let mut document = if current.trim().is_empty() {
        DocumentMut::new()
    } else {
        current
            .parse::<DocumentMut>()
            .map_err(|_| "Configuration Grok indisponible".to_string())?
    };
    match document.get("skills") {
        Some(item) if !item.is_table() => return Err("Configuration Grok indisponible".to_string()),
        None => document["skills"] = Item::Table(Table::new()),
        Some(_) => {}
    }
    let mut paths = Array::new();
    paths.push(skills);
    document["skills"]["paths"] = value(paths);
    let updated = document.to_string();
    if updated != current {
        crate::services::private_store::atomic_write(config, updated.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::configure;

    #[test]
    fn keeps_grok_settings_and_sets_only_the_cl_go_skill_path() {
        let root = tempfile::tempdir().expect("temporary Grok config");
        let config = root.path().join("config.toml");
        let skills = root.path().join("cl-go-skills");
        std::fs::create_dir(&skills).expect("skill directory");
        std::fs::write(
            &config,
            "[marketplace]\nofficial_marketplace_auto_installed = true\n\n[skills]\npaths = [\"/unsafe/global\"]\n",
        )
        .expect("initial config");

        configure(&config, &skills).expect("Grok skill config");

        let updated = std::fs::read_to_string(config).expect("updated config");
        assert!(updated.contains("official_marketplace_auto_installed = true"));
        assert!(updated.contains(skills.to_str().expect("UTF-8 test path")));
        assert!(!updated.contains("/unsafe/global"));
    }

    #[test]
    #[ignore = "requires the official Grok client"]
    fn official_grok_discovers_the_configured_cl_go_skill() {
        let binary = which::which("grok").expect("official Grok client");
        let root = tempfile::tempdir().expect("temporary Grok profile");
        let home = root.path().join("home");
        let skills = root.path().join("cl-go-skills");
        std::fs::create_dir(&home).expect("isolated home");
        std::fs::create_dir(&skills).expect("skill directory");
        std::fs::create_dir(skills.join("cl-go-test")).expect("test skill directory");
        std::fs::write(
            skills.join("cl-go-test/SKILL.md"),
            "---\nname: cl-go-test\ndescription: CL-GO isolated test skill\n---\n",
        )
        .expect("test skill");
        configure(&root.path().join("config.toml"), &skills).expect("Grok skill config");

        let output = std::process::Command::new(binary)
            .args(["inspect", "--json"])
            .current_dir(root.path())
            .env("GROK_HOME", root.path())
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .output()
            .expect("Grok inspect");

        assert!(output.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("Grok inspect JSON");
        assert!(report["skills"]
            .as_array()
            .expect("skills")
            .iter()
            .any(|skill| skill["name"] == "cl-go-test"));
    }
}
