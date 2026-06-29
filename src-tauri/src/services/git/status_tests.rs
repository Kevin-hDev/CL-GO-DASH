//! Tests du parsing numstat de status.rs (PURE).
//!
//! `parse_numstat_line` parse les lignes de `git diff HEAD --numstat` au
//! format `<add>\t<del>\t<path>`. C'est la logique critique de lecture des
//! stats de diff, extraite en fn PURE pour être testée sans subprocess git.

use super::status::parse_numstat_line;

#[test]
fn parses_standard_numstat_line() {
    let result = parse_numstat_line("12\t3\tsrc/main.rs");
    assert_eq!(result, Some(("src/main.rs".to_string(), 12, 3)));
}

#[test]
fn parses_zero_changes() {
    let result = parse_numstat_line("0\t0\tempty.txt");
    assert_eq!(result, Some(("empty.txt".to_string(), 0, 0)));
}

#[test]
fn parses_binary_file_as_zeros() {
    // git émet "-" pour les fichiers binaires (pas de diff ligne par ligne).
    // Le parseur doit l'interpréter comme 0, pas crasher.
    let result = parse_numstat_line("-\t-\timage.png");
    assert_eq!(result, Some(("image.png".to_string(), 0, 0)));
}

#[test]
fn parses_path_with_spaces() {
    let result = parse_numstat_line("5\t2\tmy file with spaces.rs");
    assert_eq!(
        result,
        Some(("my file with spaces.rs".to_string(), 5, 2))
    );
}

#[test]
fn parses_path_with_subdirectory() {
    let result = parse_numstat_line("10\t1\tsrc/deep/nested/path/file.ts");
    assert_eq!(
        result,
        Some(("src/deep/nested/path/file.ts".to_string(), 10, 1))
    );
}

#[test]
fn returns_none_for_malformed_line_too_short() {
    assert!(parse_numstat_line("12\t3").is_none());
}

#[test]
fn returns_none_for_empty_line() {
    assert!(parse_numstat_line("").is_none());
}

#[test]
fn returns_none_for_garbage() {
    assert!(parse_numstat_line("not a numstat line at all").is_none());
}

#[test]
fn handles_non_numeric_additions_as_zero() {
    // Si le champ additions n'est pas un nombre (mais pas "-"), on tolère 0.
    let result = parse_numstat_line("abc\t3\tfile.rs");
    assert_eq!(result, Some(("file.rs".to_string(), 0, 3)));
}

#[test]
fn handles_large_numbers() {
    let result = parse_numstat_line("99999\t88888\tbig.rs");
    assert_eq!(result, Some(("big.rs".to_string(), 99999, 88888)));
}
