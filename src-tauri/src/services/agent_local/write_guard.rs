use std::path::{Path, PathBuf};

const MAX_READ_PATHS: usize = 1000;
const EVICT_COUNT: usize = 100;

pub struct WriteGuard {
    read_paths: Vec<PathBuf>,
}

impl WriteGuard {
    pub fn new() -> Self {
        Self { read_paths: Vec::new() }
    }

    /// Enregistre qu'un fichier a été lu dans cette session.
    /// Collection bornée à MAX_READ_PATHS avec éviction FIFO des plus anciens.
    pub fn record_read(&mut self, path: &Path) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if self.read_paths.contains(&canonical) {
            return;
        }
        if self.read_paths.len() >= MAX_READ_PATHS {
            // Éviction FIFO : supprimer les EVICT_COUNT entrées les plus anciennes
            self.read_paths.drain(..EVICT_COUNT);
        }
        self.read_paths.push(canonical);
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
