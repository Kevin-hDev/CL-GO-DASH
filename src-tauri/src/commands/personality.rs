use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct PersonalityFile {
    pub name: String,
    pub path: String,
    pub description: String,
}

fn memory_core() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".local/share/cl-go/memory/core")
}

fn inbox_dir() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".local/share/cl-go/inbox")
}

const CORE_FILES: &[(&str, &str)] = &[
    ("identity.md", "Qui est Jackson"),
    ("me.md", "Message à soi-même"),
    ("principles.md", "Règles et valeurs"),
    ("note-to-self.md", "Notes personnelles"),
    ("user.md", "Profil de Kevin"),
];

const INBOX_FILES: &[(&str, &str)] = &[
    ("notes.md", "Notes de travail"),
    ("idea-discovery.md", "Idées en attente"),
];

#[tauri::command]
pub fn list_personality_files() -> Result<Vec<PersonalityFile>, String> {
    let core = memory_core();
    let inbox = inbox_dir();

    let mut files: Vec<PersonalityFile> = Vec::new();

    for (name, desc) in CORE_FILES {
        let path = core.join(name);
        if path.exists() {
            files.push(PersonalityFile {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
                description: desc.to_string(),
            });
        }
    }

    for (name, desc) in INBOX_FILES {
        let path = inbox.join(name);
        if path.exists() {
            files.push(PersonalityFile {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
                description: desc.to_string(),
            });
        }
    }

    Ok(files)
}

#[tauri::command]
pub fn read_personality_file(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);

    // Validate path is under allowed directories
    let core = memory_core();
    let inbox = inbox_dir();
    if !p.starts_with(&core) && !p.starts_with(&inbox) {
        return Err("Invalid path".to_string());
    }

    fs::read_to_string(&p)
        .map_err(|e| format!("Cannot read: {}", e))
}

#[tauri::command]
pub fn open_in_editor(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("File not found".to_string());
    }

    // Try MWeb first, fallback to default editor
    let mweb = Command::new("open")
        .args(["-a", "MWeb", &path])
        .spawn();

    match mweb {
        Ok(_) => Ok(()),
        Err(_) => {
            Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Cannot open: {}", e))?;
            Ok(())
        }
    }
}
