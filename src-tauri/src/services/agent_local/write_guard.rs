use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct WriteGuard {
    read_paths: HashSet<PathBuf>,
}

impl WriteGuard {
    pub fn new() -> Self {
        Self { read_paths: HashSet::new() }
    }

    /// Enregistre qu'un fichier a été lu dans cette session.
    pub fn record_read(&mut self, path: &Path) {
        if let Ok(canonical) = path.canonicalize() {
            self.read_paths.insert(canonical);
        } else {
            self.read_paths.insert(path.to_path_buf());
        }
    }

    /// Vérifie si on peut écrire dans ce fichier.
    /// OK si : le fichier n'existe pas, ou il a été lu dans cette session.
    pub fn check_write(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if self.read_paths.contains(&canonical) {
            Ok(())
        } else {
            Err("Fichier existant non lu dans cette session. Utiliser read_file d'abord.".to_string())
        }
    }
}
