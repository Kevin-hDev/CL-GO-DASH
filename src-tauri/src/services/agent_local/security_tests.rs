#[cfg(test)]
mod tests {
    use crate::services::agent_local::security::*;

    // --- check_destructive_command ---

    #[test]
    fn blocks_rm_rf_root() {
        assert!(check_destructive_command("rm -rf /").is_err());
        assert!(check_destructive_command("sudo rm -rf / --no-preserve-root").is_err());
    }

    #[test]
    fn blocks_rm_rf_wildcard() {
        assert!(check_destructive_command("rm -rf *").is_err());
    }

    #[test]
    fn blocks_sudo_rm() {
        assert!(check_destructive_command("sudo rm file.txt").is_err());
    }

    #[test]
    fn blocks_chmod_777() {
        assert!(check_destructive_command("chmod 777 file").is_err());
        assert!(check_destructive_command("chmod 777 /etc").is_err());
    }

    #[test]
    fn blocks_disk_operations() {
        assert!(check_destructive_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(check_destructive_command("mkfs.ext4 /dev/sda1").is_err());
        assert!(check_destructive_command("echo > /dev/sda").is_err());
        assert!(check_destructive_command("fdisk /dev/sda").is_err());
    }

    #[test]
    fn blocks_system_control() {
        assert!(check_destructive_command("shutdown now").is_err());
        assert!(check_destructive_command("reboot").is_err());
        assert!(check_destructive_command("init 0").is_err());
        assert!(check_destructive_command("init 6").is_err());
    }

    #[test]
    fn blocks_fork_bomb() {
        assert!(check_destructive_command(":(){:|:&};:").is_err());
    }

    #[test]
    fn blocks_eval_expansion() {
        assert!(check_destructive_command("eval $cmd").is_err());
        assert!(check_destructive_command(r#"eval "$user_input""#).is_err());
        assert!(check_destructive_command("eval  $var").is_err());
    }

    #[test]
    fn allows_safe_commands() {
        assert!(check_destructive_command("ls -la").is_ok());
        assert!(check_destructive_command("echo hello").is_ok());
        assert!(check_destructive_command("cat file.txt").is_ok());
        assert!(check_destructive_command("grep pattern *.rs").is_ok());
        assert!(check_destructive_command("eval 'echo static'").is_ok());
    }

    // --- validate_write_path ---

    #[test]
    fn write_path_allows_tmp() {
        let p = std::path::Path::new("/tmp/test-cl-go-security.txt");
        let _ = std::fs::remove_file(p);
        assert!(validate_write_path(p).is_ok());
    }

    #[test]
    fn write_path_blocks_etc() {
        let p = std::path::Path::new("/etc/evil.conf");
        assert!(validate_write_path(p).is_err());
    }

    #[test]
    fn write_path_blocks_traversal() {
        let p = std::path::Path::new("/tmp/../etc/passwd");
        assert!(validate_write_path(p).is_err());
    }

    // --- sanitize_error ---

    #[test]
    fn sanitize_masks_paths() {
        let e = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No such file or directory (os error 2): /Users/kevinh/secret",
        );
        assert_eq!(sanitize_error(e), "Fichier introuvable");
    }

    // --- validate_read_path (mission 1.1) ---

    #[test]
    fn read_path_allows_tmp() {
        let working = std::path::Path::new("/tmp");
        let p = std::path::Path::new("/tmp/some-file.txt");
        assert!(validate_read_path(p, working).is_ok());
    }

    #[test]
    fn read_path_allows_file_under_working_dir() {
        // Utilise /tmp lui-même qui existe toujours
        let working = std::path::Path::new("/tmp");
        let raw = working.join(".");
        assert!(validate_read_path(&raw, working).is_ok());
    }

    #[test]
    fn read_path_blocks_traversal_outside_home() {
        let working = std::path::Path::new("/tmp");
        let p = std::path::Path::new("/etc/passwd");
        assert!(validate_read_path(p, working).is_err());
    }

    #[test]
    fn read_path_blocks_dev() {
        let working = std::path::Path::new("/tmp");
        let p = std::path::Path::new("/dev/null");
        assert!(validate_read_path(p, working).is_err());
    }

    #[test]
    fn read_path_allows_home_subpath() {
        if let Some(home) = dirs::home_dir() {
            let target = home.join(".local/share/cl-go-dash/test.json");
            let working = std::path::Path::new("/tmp");
            assert!(validate_read_path(&target, working).is_ok());
        }
    }
}
