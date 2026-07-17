use crate::services::oauth_providers::{profile_dir, ProviderId};
use std::path::Path;
use std::sync::LazyLock;
use tokio::sync::Mutex;

const MAX_DEPTH: usize = 12;
const MAX_ENTRIES: usize = 2_048;
const MAX_TOTAL_BYTES: u64 = 32 * 1024 * 1024;
const MAX_NAME_CHARS: usize = 128;

static SYNC_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Default)]
struct CopyBudget {
    entries: usize,
    bytes: u64,
}

pub async fn prepare(provider: ProviderId) -> Result<(), String> {
    let _guard = SYNC_LOCK.lock().await;
    let data = crate::services::paths::data_dir();
    let source = data.join("skills");
    let home = profile_dir(provider).join("agent-home");
    tokio::fs::create_dir_all(&source)
        .await
        .map_err(|_| "Skills ACP indisponibles".to_string())?;
    tokio::fs::create_dir_all(&home)
        .await
        .map_err(|_| "Skills ACP indisponibles".to_string())?;
    let data = tokio::fs::canonicalize(&data)
        .await
        .map_err(|_| "Skills ACP indisponibles".to_string())?;
    let source = tokio::fs::canonicalize(&source)
        .await
        .map_err(|_| "Skills ACP indisponibles".to_string())?;
    let home = tokio::fs::canonicalize(&home)
        .await
        .map_err(|_| "Skills ACP indisponibles".to_string())?;
    if !source.starts_with(&data) || !home.starts_with(&data) {
        return Err("Skills ACP indisponibles".to_string());
    }
    tokio::task::spawn_blocking(move || {
        if provider == ProviderId::Xai {
            super::grok_skills::configure(&profile_dir(provider).join("config.toml"), &source)
        } else {
            sync_skills(&source, &home).map_err(|_| "Skills ACP indisponibles".to_string())
        }
    })
    .await
    .map_err(|_| "Skills ACP indisponibles".to_string())?
}

fn sync_skills(source: &Path, isolated_home: &Path) -> std::io::Result<()> {
    let agents = isolated_home.join(".agents");
    std::fs::create_dir_all(&agents)?;
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let staged = agents.join(format!(".skills-{nonce}.tmp"));
    let backup = agents.join(format!(".skills-{nonce}.old"));
    let current = agents.join("skills");
    let mut budget = CopyBudget::default();
    if let Err(error) = copy_tree(source, &staged, 0, &mut budget) {
        let _ = remove_path(&staged);
        return Err(error);
    }
    let had_current = std::fs::symlink_metadata(&current).is_ok();
    if had_current {
        std::fs::rename(&current, &backup)?;
    }
    if let Err(error) = std::fs::rename(&staged, &current) {
        if had_current {
            let _ = std::fs::rename(&backup, &current);
        }
        let _ = remove_path(&staged);
        return Err(error);
    }
    if had_current {
        remove_path(&backup)?;
    }
    Ok(())
}

fn copy_tree(
    source: &Path,
    destination: &Path,
    depth: usize,
    budget: &mut CopyBudget,
) -> std::io::Result<()> {
    if depth > MAX_DEPTH {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    std::fs::create_dir(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        budget.entries += 1;
        if budget.entries > MAX_ENTRIES {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        let name = entry.file_name();
        let Some(name_text) = name.to_str() else {
            return Err(std::io::ErrorKind::InvalidData.into());
        };
        if name_text.is_empty() || name_text.chars().count() > MAX_NAME_CHARS {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        let metadata = std::fs::symlink_metadata(entry.path())?;
        let target = destination.join(name);
        if metadata.file_type().is_symlink() {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        if metadata.is_dir() {
            copy_tree(&entry.path(), &target, depth + 1, budget)?;
        } else if metadata.is_file() {
            budget.bytes = budget.bytes.saturating_add(metadata.len());
            if budget.bytes > MAX_TOTAL_BYTES {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
            std::fs::copy(entry.path(), target)?;
        } else {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::sync_skills;

    #[test]
    fn mirrors_only_cl_go_skills_into_the_isolated_agent_home() {
        let root = tempfile::tempdir().expect("temporary skill bridge");
        let source = root.path().join("cl-go-skills");
        let home = root.path().join("isolated-home");
        std::fs::create_dir_all(source.join("hk-debug")).expect("CL-GO skill directory");
        std::fs::write(source.join("hk-debug/SKILL.md"), "# CL-GO skill").expect("CL-GO skill");
        std::fs::create_dir_all(home.join(".claude/skills/global-only"))
            .expect("unrelated global-like directory");
        std::fs::write(
            home.join(".claude/skills/global-only/SKILL.md"),
            "# Must stay invisible",
        )
        .expect("unrelated skill");

        sync_skills(&source, &home).expect("skill bridge");

        let exposed = home.join(".agents/skills");
        assert!(exposed.join("hk-debug/SKILL.md").is_file());
        assert!(!exposed.join("global-only").exists());
    }

    #[test]
    fn refresh_removes_skills_deleted_from_cl_go() {
        let root = tempfile::tempdir().expect("temporary skill bridge");
        let source = root.path().join("cl-go-skills");
        let home = root.path().join("isolated-home");
        std::fs::create_dir_all(source.join("old-skill")).expect("skill directory");
        std::fs::write(source.join("old-skill/SKILL.md"), "# Old").expect("skill");
        sync_skills(&source, &home).expect("first sync");
        std::fs::remove_dir_all(source.join("old-skill")).expect("remove source skill");

        sync_skills(&source, &home).expect("second sync");

        assert!(!home.join(".agents/skills/old-skill").exists());
    }
}
