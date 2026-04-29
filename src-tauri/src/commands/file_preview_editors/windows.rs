use super::DetectedEditor;
use std::path::Path;
use std::process::Command;

pub fn detect(file_path: &Path) -> Vec<DetectedEditor> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    if ext.is_empty() {
        return vec![];
    }

    let output = Command::new("cmd")
        .args(["/C", &format!("assoc {ext} 2>nul && ftype * 2>nul")])
        .output();

    let mut editors = Vec::new();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(first_line) = text.lines().next() {
            if let Some(ftype) = first_line.split('=').nth(1) {
                editors.push(DetectedEditor {
                    name: ftype.to_string(),
                    path: ftype.to_string(),
                    is_default: true,
                });
            }
        }
    }

    let known = [
        ("notepad.exe", "Notepad", "C:\\Windows\\notepad.exe"),
        ("code", "Visual Studio Code", "code"),
    ];
    for (cmd, label, path) in known {
        let exists = if path.contains('\\') {
            Path::new(path).exists()
        } else {
            Command::new("where")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };
        if exists {
            editors.push(DetectedEditor {
                name: label.to_string(),
                path: path.to_string(),
                is_default: false,
            });
        }
    }

    editors
}
