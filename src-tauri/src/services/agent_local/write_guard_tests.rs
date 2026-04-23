use std::io::Write;
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
