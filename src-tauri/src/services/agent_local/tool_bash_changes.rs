use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const MAX_SCAN_FILES: usize = 6_000;
const MAX_SCAN_DEPTH: usize = 8;
const MAX_CHANGED_PATHS: usize = 500;
const SKIPPED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
];

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileSignature {
    len: u64,
    modified_ms: u128,
}

#[derive(Default)]
pub struct FileSnapshot {
    files: BTreeMap<PathBuf, FileSignature>,
    complete: bool,
}

pub fn snapshot(root: &Path) -> FileSnapshot {
    let Ok(root) = root.canonicalize() else {
        return FileSnapshot::default();
    };
    let mut snapshot = FileSnapshot {
        files: BTreeMap::new(),
        complete: true,
    };
    scan_dir(&root, 0, &mut snapshot);
    snapshot
}

pub fn changed_paths(before: &FileSnapshot, after: &FileSnapshot) -> Vec<String> {
    if !before.complete || !after.complete {
        return Vec::new();
    }

    after
        .files
        .iter()
        .filter_map(|(path, after_sig)| match before.files.get(path) {
            Some(before_sig) if before_sig == after_sig => None,
            _ => path.to_str().map(str::to_string),
        })
        .take(MAX_CHANGED_PATHS)
        .collect()
}

fn scan_dir(root: &Path, depth: usize, snapshot: &mut FileSnapshot) {
    if depth > MAX_SCAN_DEPTH || snapshot.files.len() >= MAX_SCAN_FILES {
        snapshot.complete = false;
        return;
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if snapshot.files.len() >= MAX_SCAN_FILES {
            snapshot.complete = false;
            return;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            scan_dir(&path, depth + 1, snapshot);
        } else if file_type.is_file() {
            record_file(&path, snapshot);
        }
    }
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| SKIPPED_DIRS.contains(&name))
        .unwrap_or(false)
}

fn record_file(path: &Path, snapshot: &mut FileSnapshot) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    snapshot.files.insert(
        path.to_path_buf(),
        FileSignature {
            len: metadata.len(),
            modified_ms,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::{changed_paths, snapshot};

    #[test]
    fn detects_created_files_under_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let before = snapshot(dir.path());

        let created = dir.path().join("created.md");
        std::fs::write(&created, "hello").expect("write");
        let after = snapshot(dir.path());
        let expected = created.canonicalize().expect("canonicalize");

        assert_eq!(
            changed_paths(&before, &after),
            vec![expected.to_string_lossy().to_string()]
        );
    }

    #[test]
    fn ignores_deleted_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deleted = dir.path().join("deleted.md");
        std::fs::write(&deleted, "hello").expect("write");
        let before = snapshot(dir.path());

        std::fs::remove_file(&deleted).expect("remove");
        let after = snapshot(dir.path());

        assert!(changed_paths(&before, &after).is_empty());
    }
}
