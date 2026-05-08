use super::*;

#[test]
fn test_cwd_marker_extraction() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = format!("hello world\n{marker}\n/tmp\n");
    let (output, new_cwd) = extract_cwd(&raw, marker);
    assert_eq!(output, "hello world");
    assert_eq!(new_cwd, Some("/tmp".to_string()));
}

#[test]
fn test_cwd_marker_extraction_no_marker() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = "hello world\n";
    let (output, new_cwd) = extract_cwd(raw, marker);
    assert_eq!(output, "hello world\n");
    assert!(new_cwd.is_none());
}

#[test]
fn test_cwd_marker_spoofing_nonexistent_path() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = format!("output\n{marker}\n/nonexistent/fake/path\n");
    let (output, new_cwd) = extract_cwd(&raw, marker);
    assert_eq!(output, "output");
    assert!(new_cwd.is_none(), "Faux chemin doit être rejeté");
}

#[test]
fn test_cwd_marker_spoofing_relative_path() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = format!("output\n{marker}\nrelative/path\n");
    let (_output, new_cwd) = extract_cwd(&raw, marker);
    assert!(new_cwd.is_none(), "Chemin relatif doit être rejeté");
}

#[test]
fn test_generate_cwd_marker_unique() {
    let m1 = generate_cwd_marker();
    let m2 = generate_cwd_marker();
    assert_ne!(m1, m2, "Les markers doivent être uniques");
    assert!(m1.starts_with("<<CWD_MARKER_"));
    assert!(m1.ends_with(">>"));
}

#[test]
fn test_wrap_command_unix() {
    let marker = "<<CWD_MARKER_test>>";
    let wrapped = wrap_command_with_cwd("ls", marker);
    if !cfg!(target_os = "windows") {
        assert!(wrapped.contains("ls"));
        assert!(wrapped.contains(marker));
        assert!(wrapped.contains("pwd -P"));
    }
}

#[tokio::test]
async fn test_cwd_update_after_cd() {
    let out = execute_shell("cd /tmp", std::path::Path::new("/"), None).await;
    match out {
        Ok(shell_out) => {
            assert_eq!(
                shell_out.new_cwd.as_deref(),
                Some("/private/tmp"),
                "cd /tmp doit détecter le nouveau cwd"
            );
        }
        Err(e) => panic!("execute_shell a échoué : {e}"),
    }
}
