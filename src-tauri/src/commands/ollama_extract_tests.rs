use crate::commands::ollama_extract::safe_unpack_tar;

#[test]
fn safe_unpack_rejects_parent_dir() {
    let dir = std::env::temp_dir().join("cl-go-test-tar-traversal");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let tar_path = dir.join("evil.tar");
    {
        let mut raw = Vec::new();
        let mut header = tar::Header::new_gnu();
        let data = b"pwned";
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        // Write a path with .. directly into the raw header
        let evil = b"../escape.txt";
        header.as_gnu_mut().unwrap().name[..evil.len()].copy_from_slice(evil);
        header.set_cksum();

        use std::io::Write;
        raw.write_all(header.as_bytes()).unwrap();
        raw.write_all(data).unwrap();
        // Pad to 512-byte block
        let padding = 512 - (data.len() % 512);
        if padding < 512 {
            raw.write_all(&vec![0u8; padding]).unwrap();
        }
        // Two zero blocks = end of archive
        raw.write_all(&[0u8; 1024]).unwrap();

        std::fs::write(&tar_path, &raw).unwrap();
    }

    let dest = dir.join("output");
    std::fs::create_dir_all(&dest).unwrap();

    let file = std::fs::File::open(&tar_path).unwrap();
    let archive = tar::Archive::new(file);
    let result = safe_unpack_tar(archive, &dest);

    assert!(result.is_err(), "should reject archive with ..");
    assert!(!dir.join("escape.txt").exists(), "file should not escape dest");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn safe_unpack_rejects_symlink() {
    let dir = std::env::temp_dir().join("cl-go-test-tar-symlink");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let tar_path = dir.join("symlink.tar");
    {
        let mut raw = Vec::new();
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o777);
        header.set_entry_type(tar::EntryType::Symlink);
        let name = b"bin/ollama";
        header.as_gnu_mut().unwrap().name[..name.len()].copy_from_slice(name);
        let target = b"/usr/bin/evil";
        header.as_gnu_mut().unwrap().linkname[..target.len()].copy_from_slice(target);
        header.set_cksum();

        use std::io::Write;
        raw.write_all(header.as_bytes()).unwrap();
        raw.write_all(&[0u8; 1024]).unwrap();
        std::fs::write(&tar_path, &raw).unwrap();
    }

    let dest = dir.join("output");
    std::fs::create_dir_all(&dest).unwrap();

    let file = std::fs::File::open(&tar_path).unwrap();
    let archive = tar::Archive::new(file);
    let result = safe_unpack_tar(archive, &dest);

    assert!(result.is_err(), "should reject symlink entries");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn safe_unpack_accepts_valid_tar() {
    let dir = std::env::temp_dir().join("cl-go-test-tar-valid");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let tar_path = dir.join("good.tar");
    {
        let file = std::fs::File::create(&tar_path).unwrap();
        let mut builder = tar::Builder::new(file);
        let data = b"hello";
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "bin/ollama", &data[..])
            .unwrap();
        builder.finish().unwrap();
    }

    let dest = dir.join("output");
    std::fs::create_dir_all(&dest).unwrap();

    let file = std::fs::File::open(&tar_path).unwrap();
    let archive = tar::Archive::new(file);
    let result = safe_unpack_tar(archive, &dest);

    assert!(result.is_ok());
    assert!(dest.join("bin/ollama").exists());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn safe_unpack_rejects_hardlink() {
    let dir = std::env::temp_dir().join("cl-go-test-tar-hardlink");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let tar_path = dir.join("hardlink.tar");
    {
        let mut raw = Vec::new();
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Link);
        let name = b"bin/ollama";
        header.as_gnu_mut().unwrap().name[..name.len()].copy_from_slice(name);
        let target = b"/etc/passwd";
        header.as_gnu_mut().unwrap().linkname[..target.len()].copy_from_slice(target);
        header.set_cksum();

        use std::io::Write;
        raw.write_all(header.as_bytes()).unwrap();
        raw.write_all(&[0u8; 1024]).unwrap();
        std::fs::write(&tar_path, &raw).unwrap();
    }

    let dest = dir.join("output");
    std::fs::create_dir_all(&dest).unwrap();

    let file = std::fs::File::open(&tar_path).unwrap();
    let archive = tar::Archive::new(file);
    let result = safe_unpack_tar(archive, &dest);

    assert!(result.is_err(), "should reject hardlink entries");
    let _ = std::fs::remove_dir_all(&dir);
}
