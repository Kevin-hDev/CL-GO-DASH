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

    // --- validate_write_path (default = full disk) ---

    #[test]
    fn write_path_allows_temp_dir() {
        let tmp = std::env::temp_dir();
        let p = tmp.join("test-cl-go-security.txt");
        let _ = std::fs::remove_file(&p);
        assert!(validate_write_path(&p).is_ok());
    }

    #[test]
    fn write_path_allows_data_dir() {
        let p = crate::services::paths::data_dir().join("test.json");
        assert!(validate_write_path(&p).is_ok());
    }

    #[test]
    fn write_path_allows_app_data_dir() {
        let data = crate::services::paths::data_dir();
        let _ = std::fs::create_dir_all(&data);
        let p = data.join("write-test.json");
        assert!(validate_write_path(&p).is_ok());
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

    // --- validate_read_path (default = full disk) ---

    #[test]
    fn read_path_allows_temp() {
        let tmp = std::env::temp_dir();
        let working = &tmp;
        let p = tmp.join("some-file.txt");
        assert!(validate_read_path(&p, working).is_ok());
    }

    #[test]
    fn read_path_allows_file_under_working_dir() {
        let working = std::env::temp_dir();
        let raw = working.join(".");
        assert!(validate_read_path(&raw, &working).is_ok());
    }

    #[test]
    fn read_path_allows_app_data_dir() {
        let data = crate::services::paths::data_dir();
        let _ = std::fs::create_dir_all(&data);
        let p = data.join("read-test.json");
        assert!(validate_read_path(&p, &std::env::temp_dir()).is_ok());
    }

    #[test]
    fn read_path_allows_home_subpath() {
        if let Some(home) = dirs::home_dir() {
            let target = home.join(".local/share/cl-go-dash/test.json");
            let working = std::env::temp_dir();
            assert!(validate_read_path(&target, &working).is_ok());
        }
    }

    // --- implicit paths always allowed ---

    #[test]
    fn data_dir_always_allowed() {
        let data = crate::services::paths::data_dir();
        let _ = std::fs::create_dir_all(&data);
        let p = data.join("test-security-check.json");
        assert!(validate_read_path(&p, &std::env::temp_dir()).is_ok());
        assert!(validate_write_path(&p).is_ok());
    }

    #[test]
    fn temp_dir_always_allowed() {
        let tmp = std::env::temp_dir();
        let p = tmp.join("cl-go-test-file.txt");
        assert!(validate_read_path(&p, &tmp).is_ok());
        assert!(validate_write_path(&p).is_ok());
    }
}
