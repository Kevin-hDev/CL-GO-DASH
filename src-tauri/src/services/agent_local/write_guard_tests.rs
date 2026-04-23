use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use crate::services::agent_local::write_guard::WriteGuard;

#[test]
fn allows_write_to_new_file() {
    let guard = WriteGuard::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nouveau.txt");
    assert!(!path.exists());
    assert!(guard.check_write(&path).is_ok());
}

#[test]
fn blocks_write_to_existing_unread_file() {
    let guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "contenu existant").unwrap();
    let path = tmp.path();
    assert!(path.exists());
    let result = guard.check_write(path);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("non lu"), "message inattendu : {msg}");
}

#[test]
fn allows_write_after_read() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "contenu").unwrap();
    let path = tmp.path();
    guard.record_read(path);
    assert!(guard.check_write(path).is_ok());
}

#[test]
fn allows_edit_after_read() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "ligne existante").unwrap();
    let path = tmp.path();
    // Simule un edit_file : read d'abord, puis check_write
    guard.record_read(path);
    assert!(guard.check_write(path).is_ok());
}

#[test]
fn record_read_bounded_at_max_paths() {
    // MAX_READ_PATHS = 1000 dans write_guard.rs
    // On remplit jusqu'à la limite avec des chemins fictifs (non-canonicalisables)
    const MAX_READ_PATHS: usize = 1000;
    let mut guard = WriteGuard::new();

    // Rempli jusqu'à MAX_READ_PATHS avec des chemins fictifs
    for i in 0..MAX_READ_PATHS {
        let fake = PathBuf::from(format!("/fake/path/{i}"));
        guard.record_read(&fake);
    }

    // Le 1001ème ajout doit être ignoré silencieusement
    let extra = PathBuf::from("/fake/path/extra");
    guard.record_read(&extra); // ne doit pas paniquer

    // On vérifie que le guard ne crash pas et que check_write sur un nouveau fichier
    // (non existant) reste OK
    let new_path = PathBuf::from("/tmp/nouveau_fichier_inexistant_xyz.txt");
    assert!(!new_path.exists(), "le fichier de test ne doit pas exister");
    assert!(
        guard.check_write(&new_path).is_ok(),
        "un fichier inexistant doit toujours pouvoir être écrit"
    );
}
