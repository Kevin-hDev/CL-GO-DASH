use crate::services::agent_local::tool_files::{read_file, DEFAULT_LIMIT};
use std::io::Write;
use tempfile::NamedTempFile;

fn make_temp_file(lines: &[&str]) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    for line in lines {
        writeln!(f, "{line}").expect("write");
    }
    f
}

#[tokio::test]
async fn read_file_full() {
    let lines: Vec<&str> = (1..=10).map(|_| "hello world").collect();
    let f = make_temp_file(&lines);
    let working_dir = f.path().parent().unwrap();
    let result = read_file(
        f.path().to_str().unwrap(),
        working_dir,
        0,
        DEFAULT_LIMIT,
    )
    .await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // 10 lignes numérotées de 1 à 10
    for i in 1..=10usize {
        assert!(
            result.content.contains(&format!("{i}\thello world")),
            "ligne {i} absente"
        );
    }
    assert!(
        !result.content.contains("restante"),
        "ne doit pas avoir de message de continuation"
    );
}

#[tokio::test]
async fn read_file_offset_limit() {
    let lines: Vec<String> = (1..=20).map(|i| format!("line{i}")).collect();
    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let f = make_temp_file(&lines_ref);
    let working_dir = f.path().parent().unwrap();
    // Lire lignes 6 à 10 (offset=5, limit=5)
    let result = read_file(
        f.path().to_str().unwrap(),
        working_dir,
        5,
        5,
    )
    .await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // Doit contenir lignes 6-10 (numérotées 6..10)
    for i in 6..=10usize {
        assert!(
            result.content.contains(&format!("{i}\tline{i}")),
            "ligne {i} absente"
        );
    }
    // Ne doit pas contenir line1..5 ni line11..20
    assert!(!result.content.contains("1\tline1"), "ne doit pas contenir line1");
    assert!(!result.content.contains("11\tline11"), "ne doit pas contenir line11");
    // Message de continuation : 10 lignes restantes, offset=10
    assert!(
        result.content.contains("offset=10"),
        "doit indiquer offset=10 pour la suite"
    );
    assert!(
        result.content.contains("10 ligne(s) restante(s)"),
        "doit indiquer 10 lignes restantes"
    );
}

#[tokio::test]
async fn read_file_offset_beyond_end() {
    let lines = vec!["a", "b", "c"];
    let f = make_temp_file(&lines);
    let working_dir = f.path().parent().unwrap();
    // offset=100 dépasse la fin (3 lignes)
    let result = read_file(
        f.path().to_str().unwrap(),
        working_dir,
        100,
        DEFAULT_LIMIT,
    )
    .await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // Contenu vide (aucune ligne)
    assert!(
        result.content.trim().is_empty(),
        "le contenu doit être vide pour un offset au-delà de la fin"
    );
}

#[tokio::test]
async fn read_file_default_limit() {
    // Crée un fichier de 2500 lignes
    let lines: Vec<String> = (1..=2500).map(|i| format!("row{i}")).collect();
    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let f = make_temp_file(&lines_ref);
    let working_dir = f.path().parent().unwrap();
    let result = read_file(
        f.path().to_str().unwrap(),
        working_dir,
        0,
        DEFAULT_LIMIT,
    )
    .await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // DEFAULT_LIMIT = 2000, doit s'arrêter à la ligne 2000
    assert!(
        result.content.contains("2000\trow2000"),
        "doit contenir la ligne 2000"
    );
    assert!(
        !result.content.contains("2001\trow2001"),
        "ne doit pas contenir la ligne 2001"
    );
    // Message de continuation : 500 lignes restantes
    assert!(
        result.content.contains("500 ligne(s) restante(s)"),
        "doit indiquer 500 lignes restantes"
    );
    assert!(
        result.content.contains("offset=2000"),
        "doit indiquer offset=2000 pour la suite"
    );
}
