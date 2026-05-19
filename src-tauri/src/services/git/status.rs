use super::repo as git_repo;
use git2::StatusOptions;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

const MAX_DIRTY_FILES: usize = 200;

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

    let diff_stats = get_diff_stats(&workdir);

    let mut files = Vec::new();
    for entry in statuses.iter() {
        if files.len() >= MAX_DIRTY_FILES {
            break;
        }
        let path = entry.path().unwrap_or("?").to_string();
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

fn get_diff_stats(repo_path: &Path) -> HashMap<String, (u32, u32)> {
    let output = std::process::Command::new("git")
        .args(["-C"])
        .arg(repo_path)
        .args(["diff", "HEAD", "--numstat"])
        .output();

    let mut map = HashMap::new();
    let Ok(output) = output else { return map };
    if !output.status.success() {
        return map;
    }

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if map.len() >= MAX_DIRTY_FILES {
            break;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let add = parts[0].parse::<u32>().unwrap_or(0);
            let del = parts[1].parse::<u32>().unwrap_or(0);
            map.insert(parts[2].to_string(), (add, del));
        }
    }
    map
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
