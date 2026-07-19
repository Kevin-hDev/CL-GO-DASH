use super::repo as git_repo;
use git2::{Patch, StatusOptions};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

const MAX_DIRTY_FILES: usize = 200;
const MAX_PATH_LEN: usize = 4096;

#[derive(Debug, Clone, Serialize)]
pub struct DirtyFile {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
}

pub fn list_dirty_files(repo_path: &Path) -> Result<Vec<DirtyFile>, String> {
    let repo = git_repo::open(repo_path)?;
    let workdir = git_repo::workdir(&repo)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(false);

    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| format!("Lecture du statut : {e}"))?;

    let diff_stats = get_diff_stats(&repo);

    let mut files = Vec::new();
    for entry in statuses.iter() {
        if files.len() >= MAX_DIRTY_FILES {
            break;
        }
        let Ok(raw_path) = entry.path() else { continue };
        if raw_path.is_empty() || raw_path.len() > MAX_PATH_LEN || raw_path.contains('\0') {
            continue;
        }
        let path = raw_path.to_string();
        let st = entry.status();
        let label = if st.is_index_new() || st.is_wt_new() {
            "new"
        } else if st.is_index_modified() || st.is_wt_modified() {
            "modified"
        } else if st.is_index_deleted() || st.is_wt_deleted() {
            "deleted"
        } else {
            "changed"
        };

        let (additions, deletions) = diff_stats.get(&path).copied().unwrap_or_else(|| {
            if label == "new" {
                (count_file_lines(&workdir.join(&path)), 0)
            } else {
                (0, 0)
            }
        });

        files.push(DirtyFile {
            path,
            status: label.to_string(),
            additions,
            deletions,
        });
    }
    Ok(files)
}

fn get_diff_stats(repo: &git2::Repository) -> HashMap<String, (u32, u32)> {
    let mut map = HashMap::new();
    let Ok(tree) = repo.head().and_then(|head| head.peel_to_tree()) else { return map };
    let Ok(diff) = repo.diff_tree_to_workdir_with_index(Some(&tree), None) else { return map };
    for (index, delta) in diff.deltas().take(MAX_DIRTY_FILES).enumerate() {
        let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) else { continue };
        let Some(path) = path.to_str() else { continue };
        if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
            continue;
        }
        let stats = Patch::from_diff(&diff, index)
            .ok()
            .flatten()
            .and_then(|patch| patch.line_stats().ok());
        if let Some((_, additions, deletions)) = stats {
            map.insert(path.to_string(), (bounded_u32(additions), bounded_u32(deletions)));
        }
    }
    map
}

fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

const MAX_FILE_READ: u64 = 1_048_576;

fn count_file_lines(path: &Path) -> u32 {
    let meta = match path.symlink_metadata() {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if !meta.is_file() || meta.len() > MAX_FILE_READ {
        return 0;
    }
    std::fs::read_to_string(path)
        .map(|c| c.lines().count() as u32)
        .unwrap_or(0)
}
