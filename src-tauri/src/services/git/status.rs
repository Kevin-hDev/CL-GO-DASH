use git2::{Repository, StatusOptions};
use serde::Serialize;
use std::path::Path;

const MAX_DIRTY_FILES: usize = 200;

#[derive(Debug, Clone, Serialize)]
pub struct DirtyFile {
    pub path: String,
    pub status: String,
}

pub fn list_dirty_files(repo_path: &Path) -> Result<Vec<DirtyFile>, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Impossible d'ouvrir le dépôt : {e}"))?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| format!("Lecture du statut : {e}"))?;

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
        files.push(DirtyFile {
            path,
            status: label.to_string(),
        });
    }
    Ok(files)
}
