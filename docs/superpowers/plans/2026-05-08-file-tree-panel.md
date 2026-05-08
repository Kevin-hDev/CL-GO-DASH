# File Tree Panel — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter un panel d'arborescence de fichiers à droite du panel file-preview existant, accessible uniquement pour les sessions Projets.

**Architecture:** Panel aside indépendant (sibling du file-preview panel) avec lazy-loading d'un niveau à la fois via une commande Tauri `list_directory`. File watcher `notify` pour mise à jour temps réel. Filtre frontend en pure function. Icônes VS Code via `@iconify/icons-vscode-icons`.

**Tech Stack:** Tauri 2 (Rust), React 19, TypeScript, `notify` crate, `@iconify/icons-vscode-icons`, lucide-react (chevron), i18next

**Spec:** `docs/superpowers/specs/2026-05-08-file-tree-panel-design.md`

---

## Fichiers du plan

### Backend (créer)
- `src-tauri/src/commands/file_tree.rs` — commandes `list_directory`, `watch_project_directory`, `unwatch_project_directory`
- `src-tauri/src/models/file_tree.rs` — struct `FileEntry`

### Backend (modifier)
- `src-tauri/src/commands/mod.rs` — ajouter `pub mod file_tree` + `pub use file_tree::*`
- `src-tauri/src/models/mod.rs` — ajouter `pub mod file_tree` + `pub use file_tree::*`
- `src-tauri/src/lib.rs` — enregistrer les 3 nouvelles commandes dans `invoke_handler!`

### Frontend (créer)
- `src/lib/file-tree-hidden.ts` — liste des dossiers/fichiers masqués
- `src/lib/file-tree-filter.ts` — logique de filtrage pure
- `src/hooks/use-file-tree.ts` — état complet du tree panel
- `src/components/file-tree/file-tree-filter.tsx` — barre de recherche
- `src/components/file-tree/file-tree-node.tsx` — noeud récursif (dossier/fichier)
- `src/components/file-tree/file-tree-panel.tsx` — panel aside container
- `src/components/file-tree/file-tree-panel.css` — styles colocalisés

### Frontend (modifier)
- `src/components/file-preview/file-preview-panel.tsx` — ajouter le bouton dossier dans le header
- `src/components/agent-local/agent-chat-detail.tsx` — ajouter `FileTreePanel` comme sibling

### Tests (créer)
- `src/lib/file-tree-hidden.test.ts`
- `src/lib/file-tree-filter.test.ts`
- `src-tauri/src/commands/file_tree_tests.rs`
- `src-tauri/src/models/file_tree_tests.rs`

### i18n (modifier)
- `src/i18n/fr.json`, `en.json`, `es.json`, `de.json`, `it.json`, `zh.json`, `ja.json` — ajouter clé `"fileTree"`

---

## Task 1 : Backend — struct FileEntry + commande list_directory

**Files:**
- Create: `src-tauri/src/models/file_tree.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/commands/file_tree.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs:106-243`
- Test: `src-tauri/src/models/file_tree_tests.rs`
- Test: `src-tauri/src/commands/file_tree_tests.rs`

### Étape 1 — Écrire les tests du modèle FileEntry

- [ ] **Créer `src-tauri/src/models/file_tree_tests.rs`**

```rust
#[cfg(test)]
mod tests {
    use crate::models::file_tree::FileEntry;

    #[test]
    fn serialize_file_entry() {
        let entry = FileEntry {
            name: "main.rs".into(),
            path: "/project/src/main.rs".into(),
            is_dir: false,
            extension: Some("rs".into()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"name\":\"main.rs\""));
        assert!(json.contains("\"is_dir\":false"));
        assert!(json.contains("\"extension\":\"rs\""));
    }

    #[test]
    fn serialize_dir_entry() {
        let entry = FileEntry {
            name: "src".into(),
            path: "/project/src".into(),
            is_dir: true,
            extension: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"is_dir\":true"));
        assert!(json.contains("\"extension\":null"));
    }

    #[test]
    fn extension_tar_gz() {
        let entry = FileEntry {
            name: "archive.tar.gz".into(),
            path: "/project/archive.tar.gz".into(),
            is_dir: false,
            extension: Some("gz".into()),
        };
        assert_eq!(entry.extension.as_deref(), Some("gz"));
    }

    #[test]
    fn dotfile_no_extension() {
        let entry = FileEntry {
            name: ".env".into(),
            path: "/project/.env".into(),
            is_dir: false,
            extension: None,
        };
        assert_eq!(entry.extension, None);
    }
}
```

- [ ] **Vérifier que le test échoue**

Run: `cd src-tauri && cargo test file_tree_tests -- --nocapture 2>&1 | head -20`
Expected: erreur de compilation (module pas encore créé)

### Étape 2 — Implémenter le modèle FileEntry

- [ ] **Créer `src-tauri/src/models/file_tree.rs`**

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
}
```

- [ ] **Modifier `src-tauri/src/models/mod.rs`**

Ajouter après `pub use config::*;` :

```rust
pub mod file_tree;
#[cfg(test)]
mod file_tree_tests;

pub use file_tree::*;
```

- [ ] **Vérifier que les tests du modèle passent**

Run: `cd src-tauri && cargo test file_tree_tests -- --nocapture`
Expected: 4 tests passed

- [ ] **Commit**

```bash
git add src-tauri/src/models/file_tree.rs src-tauri/src/models/file_tree_tests.rs src-tauri/src/models/mod.rs
git commit -m "feat(backend): add FileEntry model for file tree"
```

### Étape 3 — Écrire les tests de list_directory

- [ ] **Créer `src-tauri/src/commands/file_tree_tests.rs`**

```rust
#[cfg(test)]
mod tests {
    use crate::commands::file_tree::{list_directory, HIDDEN_ENTRIES};
    use std::fs;
    use tempfile::TempDir;

    fn setup_tree() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("docs")).unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        fs::create_dir(root.join("node_modules")).unwrap();
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("README.md"), "hello").unwrap();
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join(".env"), "SECRET=x").unwrap();
        fs::write(root.join(".DS_Store"), "").unwrap();
        tmp
    }

    #[tokio::test]
    async fn sorted_dirs_first_then_files() {
        let tmp = setup_tree();
        let entries = list_directory(tmp.path().to_str().unwrap().into(), false)
            .await
            .unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        let first_file_idx = names.iter().position(|n| !entries[names.iter().position(|x| x == n).unwrap()].is_dir);
        let last_dir_idx = names.iter().rposition(|n| entries[names.iter().position(|x| x == n).unwrap()].is_dir);
        if let (Some(first_file), Some(last_dir)) = (first_file_idx, last_dir_idx) {
            assert!(last_dir < first_file, "dirs must come before files");
        }
    }

    #[tokio::test]
    async fn hides_git_but_keeps_node_modules() {
        let tmp = setup_tree();
        let entries = list_directory(tmp.path().to_str().unwrap().into(), false)
            .await
            .unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(!names.contains(&".git"), ".git should be hidden");
        assert!(!names.contains(&"target"), "target should be hidden");
        assert!(!names.contains(&".DS_Store"), ".DS_Store should be hidden");
        assert!(names.contains(&"node_modules"), "node_modules should be visible");
        assert!(names.contains(&".env"), ".env should be visible");
    }

    #[tokio::test]
    async fn show_hidden_includes_git() {
        let tmp = setup_tree();
        let entries = list_directory(tmp.path().to_str().unwrap().into(), true)
            .await
            .unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&".git"), ".git should be visible with show_hidden");
    }

    #[tokio::test]
    async fn nonexistent_dir_returns_error() {
        let result = list_directory("/nonexistent/path/12345".into(), false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn path_traversal_blocked() {
        let tmp = setup_tree();
        let evil_path = format!("{}/../../../etc", tmp.path().to_str().unwrap());
        let result = list_directory(evil_path, false).await;
        assert!(result.is_err());
    }

    #[test]
    fn hidden_entries_contains_expected() {
        assert!(HIDDEN_ENTRIES.contains(&".git"));
        assert!(HIDDEN_ENTRIES.contains(&"target"));
        assert!(HIDDEN_ENTRIES.contains(&".DS_Store"));
        assert!(!HIDDEN_ENTRIES.contains(&"node_modules"));
    }
}
```

- [ ] **Vérifier que les tests échouent**

Run: `cd src-tauri && cargo test file_tree_tests -- --nocapture 2>&1 | head -20`
Expected: erreur de compilation (module `file_tree` pas dans commands)

### Étape 4 — Implémenter list_directory

- [ ] **Créer `src-tauri/src/commands/file_tree.rs`**

```rust
use crate::models::file_tree::FileEntry;
use std::path::{Component, Path};

const MAX_ENTRIES: usize = 5000;
const MAX_PATH_LEN: usize = 4096;

pub const HIDDEN_ENTRIES: &[&str] = &[
    ".git",
    ".DS_Store",
    ".next",
    ".turbo",
    "__pycache__",
    "dist",
    "target",
    "build",
    ".cache",
];

fn validate_dir_path(path: &str) -> Result<std::path::PathBuf, String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
        return Err("Chemin invalide".into());
    }
    if Path::new(path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Chemin invalide".into());
    }
    let canonical = std::fs::canonicalize(path).map_err(|_| "Dossier introuvable".to_string())?;
    if !canonical.is_dir() {
        return Err("Dossier introuvable".into());
    }
    Ok(canonical)
}

#[tauri::command]
pub async fn list_directory(path: String, show_hidden: bool) -> Result<Vec<FileEntry>, String> {
    let canonical = validate_dir_path(&path)?;

    let mut dirs: Vec<FileEntry> = Vec::new();
    let mut files: Vec<FileEntry> = Vec::new();

    let read_dir =
        std::fs::read_dir(&canonical).map_err(|_| "Impossible de lire ce dossier".to_string())?;

    let mut count = 0usize;
    for entry in read_dir.flatten() {
        if count >= MAX_ENTRIES {
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();

        if !show_hidden && HIDDEN_ENTRIES.contains(&name.as_str()) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        let is_dir = file_type.is_dir();
        let extension = if is_dir {
            None
        } else {
            Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
        };

        let entry_path = entry.path().to_string_lossy().into_owned();

        let fe = FileEntry {
            name,
            path: entry_path,
            is_dir,
            extension,
        };

        if is_dir {
            dirs.push(fe);
        } else {
            files.push(fe);
        }
        count += 1;
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    dirs.append(&mut files);
    Ok(dirs)
}
```

- [ ] **Modifier `src-tauri/src/commands/mod.rs`** — ajouter après la ligne `pub mod file_preview_office;` :

```rust
pub mod file_tree;
#[cfg(test)]
mod file_tree_tests;
```

Et après `pub use file_preview_office::*;` :

```rust
pub use file_tree::*;
```

- [ ] **Modifier `src-tauri/src/lib.rs`** — dans `invoke_handler!`, ajouter après le commentaire `// File preview` (ligne 200) :

```rust
            // File tree
            commands::list_directory,
```

- [ ] **Ajouter `tempfile` aux dev-dependencies** dans `src-tauri/Cargo.toml` si absent :

Run: `cd src-tauri && grep -q 'tempfile' Cargo.toml && echo "already present" || echo 'tempfile = "3"' >> Cargo.toml`

- [ ] **Vérifier que les tests passent**

Run: `cd src-tauri && cargo test file_tree_tests -- --nocapture`
Expected: tous les tests passent

- [ ] **Vérifier que le projet compile**

Run: `cd src-tauri && cargo check`
Expected: pas d'erreur

- [ ] **Commit**

```bash
git add src-tauri/src/commands/file_tree.rs src-tauri/src/commands/file_tree_tests.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat(backend): add list_directory command with tests"
```

---

## Task 2 : Backend — file watcher pour le tree

**Files:**
- Modify: `src-tauri/src/commands/file_tree.rs`
- Modify: `src-tauri/src/lib.rs`

### Étape 1 — Ajouter watch/unwatch dans file_tree.rs

- [ ] **Ajouter en haut de `src-tauri/src/commands/file_tree.rs`** les imports :

```rust
use notify::{RecursiveMode, Watcher};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
```

- [ ] **Ajouter la struct de state et les commandes watch/unwatch** à la fin du fichier :

```rust
pub struct FileTreeWatcher {
    inner: Mutex<Option<WatcherState>>,
}

struct WatcherState {
    _watcher: notify::RecommendedWatcher,
    path: String,
}

impl FileTreeWatcher {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct FileTreeChangedPayload {
    path: String,
    kind: String,
}

#[tauri::command]
pub fn watch_project_directory(
    path: String,
    app: AppHandle,
    state: State<'_, FileTreeWatcher>,
) -> Result<(), String> {
    let canonical = validate_dir_path(&path)?;
    let mut guard = state.inner.lock().map_err(|_| "Lock error".to_string())?;

    // Stop existing watcher
    *guard = None;

    let handle = app.clone();
    let watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        if let Ok(event) = res {
            let kind = format!("{:?}", event.kind);
            for changed in &event.paths {
                let parent = changed
                    .parent()
                    .unwrap_or(changed)
                    .to_string_lossy()
                    .into_owned();
                let _ = handle.emit(
                    "file-tree-changed",
                    FileTreeChangedPayload {
                        path: parent,
                        kind: kind.clone(),
                    },
                );
            }
        }
    })
    .map_err(|e| format!("Watcher error: {e}"))?;

    let mut w = watcher;
    w.watch(&canonical, RecursiveMode::Recursive)
        .map_err(|e| format!("Watch error: {e}"))?;

    *guard = Some(WatcherState {
        _watcher: w,
        path: path.clone(),
    });

    Ok(())
}

#[tauri::command]
pub fn unwatch_project_directory(state: State<'_, FileTreeWatcher>) -> Result<(), String> {
    let mut guard = state.inner.lock().map_err(|_| "Lock error".to_string())?;
    *guard = None;
    Ok(())
}
```

- [ ] **Modifier `src-tauri/src/lib.rs`** — ajouter le state management et les commandes :

Après les `.manage(...)` existants (vers ligne 42), ajouter :

```rust
            .manage(commands::file_tree::FileTreeWatcher::new())
```

Dans `invoke_handler!`, après `commands::list_directory,` :

```rust
            commands::watch_project_directory,
            commands::unwatch_project_directory,
```

- [ ] **Vérifier la compilation**

Run: `cd src-tauri && cargo check`
Expected: pas d'erreur

- [ ] **Commit**

```bash
git add src-tauri/src/commands/file_tree.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add file tree watcher (watch/unwatch project directory)"
```

---

## Task 3 : Frontend — lib utilitaires (hidden + filter + tests)

**Files:**
- Create: `src/lib/file-tree-hidden.ts`
- Create: `src/lib/file-tree-filter.ts`
- Test: `src/lib/file-tree-hidden.test.ts`
- Test: `src/lib/file-tree-filter.test.ts`

### Étape 1 — Écrire les tests de file-tree-hidden

- [ ] **Créer `src/lib/file-tree-hidden.test.ts`**

```typescript
import { describe, expect, it } from "vitest";
import { isHiddenEntry } from "./file-tree-hidden";

describe("isHiddenEntry", () => {
  it("hides .git", () => {
    expect(isHiddenEntry(".git")).toBe(true);
  });

  it("hides target", () => {
    expect(isHiddenEntry("target")).toBe(true);
  });

  it("hides .DS_Store", () => {
    expect(isHiddenEntry(".DS_Store")).toBe(true);
  });

  it("hides __pycache__", () => {
    expect(isHiddenEntry("__pycache__")).toBe(true);
  });

  it("keeps node_modules visible", () => {
    expect(isHiddenEntry("node_modules")).toBe(false);
  });

  it("keeps .env visible", () => {
    expect(isHiddenEntry(".env")).toBe(false);
  });

  it("keeps src visible", () => {
    expect(isHiddenEntry("src")).toBe(false);
  });

  it("keeps README.md visible", () => {
    expect(isHiddenEntry("README.md")).toBe(false);
  });
});
```

- [ ] **Vérifier que le test échoue**

Run: `npx vitest run src/lib/file-tree-hidden.test.ts 2>&1 | tail -10`
Expected: erreur import (module pas encore créé)

### Étape 2 — Implémenter file-tree-hidden

- [ ] **Créer `src/lib/file-tree-hidden.ts`**

```typescript
const HIDDEN_ENTRIES = new Set([
  ".git",
  ".DS_Store",
  ".next",
  ".turbo",
  "__pycache__",
  "dist",
  "target",
  "build",
  ".cache",
]);

export function isHiddenEntry(name: string): boolean {
  return HIDDEN_ENTRIES.has(name);
}
```

- [ ] **Vérifier que les tests passent**

Run: `npx vitest run src/lib/file-tree-hidden.test.ts`
Expected: 8 tests passed

- [ ] **Commit**

```bash
git add src/lib/file-tree-hidden.ts src/lib/file-tree-hidden.test.ts
git commit -m "feat(lib): add file-tree-hidden with tests"
```

### Étape 3 — Écrire les tests de file-tree-filter

- [ ] **Créer `src/lib/file-tree-filter.test.ts`**

```typescript
import { describe, expect, it } from "vitest";
import { filterTree } from "./file-tree-filter";
import type { FileEntry } from "./file-tree-filter";

const TREE: FileEntry[] = [
  { name: "src", path: "/p/src", is_dir: true, extension: null },
  { name: "docs", path: "/p/docs", is_dir: true, extension: null },
  { name: "README.md", path: "/p/README.md", is_dir: false, extension: "md" },
  { name: "main.rs", path: "/p/main.rs", is_dir: false, extension: "rs" },
];

const CHILDREN = new Map<string, FileEntry[]>([
  [
    "/p/src",
    [
      { name: "lib.rs", path: "/p/src/lib.rs", is_dir: false, extension: "rs" },
      { name: "utils", path: "/p/src/utils", is_dir: true, extension: null },
    ],
  ],
  [
    "/p/src/utils",
    [
      { name: "helpers.rs", path: "/p/src/utils/helpers.rs", is_dir: false, extension: "rs" },
    ],
  ],
]);

describe("filterTree", () => {
  it("returns all entries on empty query", () => {
    const result = filterTree(TREE, CHILDREN, "");
    expect(result.entries).toEqual(TREE);
    expect(result.expandedPaths.size).toBe(0);
  });

  it("matches partial case-insensitive", () => {
    const result = filterTree(TREE, CHILDREN, "read");
    expect(result.entries.map((e) => e.name)).toContain("README.md");
  });

  it("auto-expands parents when child matches", () => {
    const result = filterTree(TREE, CHILDREN, "helpers");
    expect(result.expandedPaths.has("/p/src")).toBe(true);
    expect(result.expandedPaths.has("/p/src/utils")).toBe(true);
  });

  it("includes parent dirs that contain matches", () => {
    const result = filterTree(TREE, CHILDREN, "lib.rs");
    expect(result.entries.map((e) => e.name)).toContain("src");
  });

  it("handles accented characters", () => {
    const entries: FileEntry[] = [
      { name: "résumé.md", path: "/p/résumé.md", is_dir: false, extension: "md" },
    ];
    const result = filterTree(entries, new Map(), "résu");
    expect(result.entries).toHaveLength(1);
  });

  it("handles spaces in query", () => {
    const entries: FileEntry[] = [
      { name: "my file.ts", path: "/p/my file.ts", is_dir: false, extension: "ts" },
    ];
    const result = filterTree(entries, new Map(), "my fi");
    expect(result.entries).toHaveLength(1);
  });
});
```

- [ ] **Vérifier que le test échoue**

Run: `npx vitest run src/lib/file-tree-filter.test.ts 2>&1 | tail -10`
Expected: erreur import

### Étape 4 — Implémenter file-tree-filter

- [ ] **Créer `src/lib/file-tree-filter.ts`**

```typescript
export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  extension: string | null;
}

export interface FilterResult {
  entries: FileEntry[];
  expandedPaths: Set<string>;
}

export function filterTree(
  rootEntries: FileEntry[],
  childrenMap: Map<string, FileEntry[]>,
  query: string,
): FilterResult {
  if (!query.trim()) {
    return { entries: rootEntries, expandedPaths: new Set() };
  }

  const lower = query.toLowerCase();
  const expandedPaths = new Set<string>();
  const matchingPaths = new Set<string>();

  function searchRecursive(entries: FileEntry[], ancestors: string[]): boolean {
    let anyMatch = false;
    for (const entry of entries) {
      const nameMatches = entry.name.toLowerCase().includes(lower);
      let childMatch = false;

      if (entry.is_dir) {
        const children = childrenMap.get(entry.path) ?? [];
        childMatch = searchRecursive(children, [...ancestors, entry.path]);
      }

      if (nameMatches || childMatch) {
        matchingPaths.add(entry.path);
        anyMatch = true;
        if (childMatch) {
          expandedPaths.add(entry.path);
        }
        for (const ancestor of ancestors) {
          expandedPaths.add(ancestor);
          matchingPaths.add(ancestor);
        }
      }
    }
    return anyMatch;
  }

  searchRecursive(rootEntries, []);

  const filtered = rootEntries.filter((e) => matchingPaths.has(e.path));
  return { entries: filtered, expandedPaths };
}
```

- [ ] **Vérifier que les tests passent**

Run: `npx vitest run src/lib/file-tree-filter.test.ts`
Expected: 6 tests passed

- [ ] **Commit**

```bash
git add src/lib/file-tree-filter.ts src/lib/file-tree-filter.test.ts
git commit -m "feat(lib): add file-tree-filter with tests"
```

---

## Task 4 : i18n — 7 langues

**Files:**
- Modify: `src/i18n/fr.json`, `en.json`, `es.json`, `de.json`, `it.json`, `zh.json`, `ja.json`

### Étape 1 — Ajouter les clés fileTree dans les 7 fichiers

- [ ] **Ajouter le bloc `"fileTree"` dans chaque fichier** (après le bloc `"filePreview"`):

**`fr.json`** :
```json
"fileTree": {
  "filterPlaceholder": "Filtrer les fichiers...",
  "emptyDirectory": "Ce dossier est vide",
  "tooManyEntries": "Trop d'éléments ({{count}} max)",
  "toggleTree": "Arborescence du projet",
  "loadError": "Impossible de lire ce dossier",
  "noResults": "Aucun résultat"
}
```

**`en.json`** :
```json
"fileTree": {
  "filterPlaceholder": "Filter files...",
  "emptyDirectory": "This folder is empty",
  "tooManyEntries": "Too many entries ({{count}} max)",
  "toggleTree": "Project file tree",
  "loadError": "Could not read this folder",
  "noResults": "No results"
}
```

**`es.json`** :
```json
"fileTree": {
  "filterPlaceholder": "Filtrar archivos...",
  "emptyDirectory": "Esta carpeta está vacía",
  "tooManyEntries": "Demasiados elementos ({{count}} máx)",
  "toggleTree": "Árbol de archivos del proyecto",
  "loadError": "No se pudo leer esta carpeta",
  "noResults": "Sin resultados"
}
```

**`de.json`** :
```json
"fileTree": {
  "filterPlaceholder": "Dateien filtern...",
  "emptyDirectory": "Dieser Ordner ist leer",
  "tooManyEntries": "Zu viele Einträge (max. {{count}})",
  "toggleTree": "Projektdateibaum",
  "loadError": "Ordner konnte nicht gelesen werden",
  "noResults": "Keine Ergebnisse"
}
```

**`it.json`** :
```json
"fileTree": {
  "filterPlaceholder": "Filtra file...",
  "emptyDirectory": "Questa cartella è vuota",
  "tooManyEntries": "Troppi elementi (max {{count}})",
  "toggleTree": "Albero dei file del progetto",
  "loadError": "Impossibile leggere questa cartella",
  "noResults": "Nessun risultato"
}
```

**`zh.json`** :
```json
"fileTree": {
  "filterPlaceholder": "筛选文件...",
  "emptyDirectory": "此文件夹为空",
  "tooManyEntries": "条目过多（最多{{count}}）",
  "toggleTree": "项目文件树",
  "loadError": "无法读取此文件夹",
  "noResults": "无结果"
}
```

**`ja.json`** :
```json
"fileTree": {
  "filterPlaceholder": "ファイルを検索...",
  "emptyDirectory": "このフォルダは空です",
  "tooManyEntries": "エントリが多すぎます（最大{{count}}）",
  "toggleTree": "プロジェクトファイルツリー",
  "loadError": "このフォルダを読み取れません",
  "noResults": "結果なし"
}
```

- [ ] **Vérifier le TS**

Run: `npx tsc --noEmit`
Expected: pas d'erreur

- [ ] **Commit**

```bash
git add src/i18n/*.json
git commit -m "feat(i18n): add fileTree keys in 7 languages"
```

---

## Task 5 : Frontend — hook use-file-tree

**Files:**
- Create: `src/hooks/use-file-tree.ts`

### Étape 1 — Créer le hook

- [ ] **Créer `src/hooks/use-file-tree.ts`**

```typescript
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { FileEntry } from "@/lib/file-tree-filter";
import { isHiddenEntry } from "@/lib/file-tree-hidden";

const DEFAULT_WIDTH = 240;
const MIN_WIDTH = 240;
const MAX_WIDTH = 500;
const MAX_EXPANDED = 500;

function treeStorageKey(sessionId: string | null): string {
  return `clgo-file-tree-width:${sessionId ?? "none"}`;
}

function readStoredWidth(sessionId: string | null): number {
  try {
    const raw = localStorage.getItem(treeStorageKey(sessionId));
    const parsed = Number(raw);
    if (parsed >= MIN_WIDTH && parsed <= MAX_WIDTH) return parsed;
  } catch { /* ignore */ }
  return DEFAULT_WIDTH;
}

export function useFileTree(sessionId: string | null, projectPath: string | undefined) {
  const [open, setOpen] = useState(false);
  const [width, setWidth] = useState(() => readStoredWidth(sessionId));
  const [resizing, setResizing] = useState(false);
  const resizeRef = useRef<{ startX: number; startWidth: number } | null>(null);

  const [rootEntries, setRootEntries] = useState<FileEntry[]>([]);
  const [childrenMap, setChildrenMap] = useState<Map<string, FileEntry[]>>(new Map());
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState("");
  const [loadError, setLoadError] = useState<string | null>(null);

  const hasProject = !!projectPath;

  const loadDirectory = useCallback(async (dirPath: string): Promise<FileEntry[]> => {
    const entries = await invoke<FileEntry[]>("list_directory", {
      path: dirPath,
      showHidden: false,
    });
    return entries;
  }, []);

  useEffect(() => {
    if (!projectPath || !open) return;
    setLoadError(null);
    loadDirectory(projectPath)
      .then(setRootEntries)
      .catch(() => setLoadError("error"));
  }, [projectPath, open, loadDirectory]);

  useEffect(() => {
    if (!projectPath || !open) return;
    invoke("watch_project_directory", { path: projectPath }).catch(() => {});
    return () => {
      invoke("unwatch_project_directory").catch(() => {});
    };
  }, [projectPath, open]);

  useEffect(() => {
    if (!open) return;
    const unlisten = listen<{ path: string }>("file-tree-changed", (event) => {
      const changedDir = event.payload.path;
      if (changedDir === projectPath) {
        loadDirectory(changedDir).then(setRootEntries).catch(() => {});
      } else if (expandedPaths.has(changedDir)) {
        loadDirectory(changedDir).then((entries) => {
          setChildrenMap((prev) => {
            const next = new Map(prev);
            next.set(changedDir, entries);
            return next;
          });
        }).catch(() => {});
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [open, projectPath, expandedPaths, loadDirectory]);

  const toggleExpand = useCallback(async (dirPath: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(dirPath)) {
        next.delete(dirPath);
        return next;
      }
      if (next.size >= MAX_EXPANDED) return prev;
      next.add(dirPath);
      return next;
    });

    if (!childrenMap.has(dirPath)) {
      const entries = await loadDirectory(dirPath).catch(() => [] as FileEntry[]);
      setChildrenMap((prev) => {
        const next = new Map(prev);
        next.set(dirPath, entries);
        return next;
      });
    }
  }, [childrenMap, loadDirectory]);

  const toggleOpen = useCallback(() => {
    setOpen((v) => !v);
  }, []);

  const closeTree = useCallback(() => {
    setOpen(false);
  }, []);

  // Persist width
  useEffect(() => {
    localStorage.setItem(treeStorageKey(sessionId), String(width));
  }, [sessionId, width]);

  // Resize logic
  const startResize = useCallback((event: React.PointerEvent) => {
    event.preventDefault();
    resizeRef.current = { startX: event.clientX, startWidth: width };
    setResizing(true);
  }, [width]);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startX - event.clientX;
      setWidth(Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, resizeRef.current.startWidth + delta)));
    };
    const onUp = () => {
      resizeRef.current = null;
      setResizing(false);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, []);

  // Reset on session change
  useEffect(() => {
    setOpen(false);
    setRootEntries([]);
    setChildrenMap(new Map());
    setExpandedPaths(new Set());
    setFilter("");
    setLoadError(null);
  }, [sessionId]);

  return {
    open,
    width,
    resizing,
    hasProject,
    rootEntries,
    childrenMap,
    expandedPaths,
    filter,
    loadError,
    setFilter,
    toggleOpen,
    closeTree,
    toggleExpand,
    startResize,
  };
}
```

- [ ] **Vérifier le TS**

Run: `npx tsc --noEmit`
Expected: pas d'erreur

- [ ] **Commit**

```bash
git add src/hooks/use-file-tree.ts
git commit -m "feat(hooks): add use-file-tree hook"
```

---

## Task 6 : Frontend — composants file-tree (filter, node, panel, CSS)

**Files:**
- Create: `src/components/file-tree/file-tree-filter.tsx`
- Create: `src/components/file-tree/file-tree-node.tsx`
- Create: `src/components/file-tree/file-tree-panel.tsx`
- Create: `src/components/file-tree/file-tree-panel.css`

### Étape 1 — Créer le CSS

- [ ] **Vérifier absence de collision de préfixe `ft-`**

Run: `grep -rn "\.ft-" src/ --include="*.css" | head -5`
Expected: aucun résultat

- [ ] **Créer `src/components/file-tree/file-tree-panel.css`**

```css
.ft-panel {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  width: 0 !important;
  height: 100%;
  overflow: hidden;
  border-left: 1px solid var(--edge);
  background: var(--void);
  transform: translateX(100%);
  opacity: 0;
  pointer-events: none;
  flex-shrink: 0;
  transition:
    transform 300ms cubic-bezier(0.4, 0, 0.2, 1),
    width 300ms cubic-bezier(0.4, 0, 0.2, 1),
    opacity 220ms ease;
}

.ft-panel.open {
  width: var(--ft-width) !important;
  transform: translateX(0);
  opacity: 1;
  pointer-events: auto;
}

.ft-panel.resizing {
  transition: none;
}

.ft-resize {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 6px;
  cursor: ew-resize;
  z-index: 2;
}

.ft-resize:hover {
  background: var(--edge);
}

.ft-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--edge);
  flex-shrink: 0;
}

.ft-filter-input {
  flex: 1;
  min-width: 0;
  padding: 6px 10px;
  border: 1px solid var(--edge);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--ink);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  outline: none;
}

.ft-filter-input::placeholder {
  color: var(--ink-faint);
}

.ft-filter-input:focus {
  border-color: var(--pulse);
}

.ft-filter-wrap {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
}

.ft-filter-clear {
  position: absolute;
  right: 6px;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: var(--ink-faint);
  cursor: pointer;
  padding: 0;
}

.ft-filter-clear:hover {
  color: var(--ink);
}

.ft-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: thin;
  scrollbar-color: var(--ink-faint) transparent;
  padding: 4px 0;
}

.ft-node {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--ink);
  padding-right: 12px;
}

.ft-node:hover {
  background: var(--surface-hover);
}

.ft-node-active {
  background: var(--surface-hover);
  color: var(--pulse);
}

.ft-chevron {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: transform 200ms ease-out;
  color: var(--ink-faint);
}

.ft-chevron.expanded {
  transform: rotate(90deg);
}

.ft-chevron-placeholder {
  width: 16px;
  flex-shrink: 0;
}

.ft-children {
  overflow: hidden;
  transition: max-height 200ms ease-out, opacity 150ms ease-out;
}

.ft-children.collapsed {
  max-height: 0;
  opacity: 0;
}

.ft-empty {
  padding: 16px;
  text-align: center;
  color: var(--ink-faint);
  font-size: var(--text-xs);
}
```

### Étape 2 — Créer le composant FileTreeFilter

- [ ] **Créer `src/components/file-tree/file-tree-filter.tsx`**

```tsx
import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { Search, X } from "lucide-react";

interface FileTreeFilterProps {
  value: string;
  onChange: (value: string) => void;
}

export function FileTreeFilter({ value, onChange }: FileTreeFilterProps) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onChange("");
      inputRef.current?.blur();
    }
  };

  return (
    <div className="ft-filter-wrap">
      <Search size={13} style={{ position: "absolute", left: 10, color: "var(--ink-faint)" }} />
      <input
        ref={inputRef}
        className="ft-filter-input"
        style={{ paddingLeft: 30 }}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t("fileTree.filterPlaceholder")}
        spellCheck={false}
      />
      {value && (
        <button className="ft-filter-clear" onClick={() => onChange("")} type="button">
          <X size={13} />
        </button>
      )}
    </div>
  );
}
```

### Étape 3 — Créer le composant FileTreeNode

- [ ] **Créer `src/components/file-tree/file-tree-node.tsx`**

```tsx
import { useEffect, useRef, useState } from "react";
import { ChevronRight } from "lucide-react";
import { FileIcon } from "@/components/file-preview/file-icon";
import { Icon } from "@iconify/react";
import defaultFolder from "@iconify/icons-vscode-icons/default-folder.js";
import defaultFolderOpened from "@iconify/icons-vscode-icons/default-folder-opened.js";
import type { FileEntry } from "@/lib/file-tree-filter";

interface FileTreeNodeProps {
  entry: FileEntry;
  depth: number;
  expanded: boolean;
  active: boolean;
  children?: FileEntry[];
  expandedPaths: Set<string>;
  childrenMap: Map<string, FileEntry[]>;
  activePath: string | null;
  onToggle: (path: string) => void;
  onSelect: (path: string) => void;
}

export function FileTreeNode({
  entry,
  depth,
  expanded,
  active,
  children,
  expandedPaths,
  childrenMap,
  activePath,
  onToggle,
  onSelect,
}: FileTreeNodeProps) {
  const childrenRef = useRef<HTMLDivElement>(null);
  const [maxHeight, setMaxHeight] = useState<string>(expanded ? "none" : "0");

  useEffect(() => {
    if (!childrenRef.current) return;
    if (expanded) {
      const h = childrenRef.current.scrollHeight;
      setMaxHeight(`${h}px`);
      const timer = setTimeout(() => setMaxHeight("none"), 200);
      return () => clearTimeout(timer);
    }
    const h = childrenRef.current.scrollHeight;
    setMaxHeight(`${h}px`);
    requestAnimationFrame(() => setMaxHeight("0"));
  }, [expanded]);

  const handleClick = () => {
    if (entry.is_dir) {
      onToggle(entry.path);
    } else {
      onSelect(entry.path);
    }
  };

  const folderIcon = expanded ? defaultFolderOpened : defaultFolder;

  return (
    <div>
      <div
        className={`ft-node ${active ? "ft-node-active" : ""}`}
        style={{ paddingLeft: depth * 16 + 8 }}
        onClick={handleClick}
      >
        {entry.is_dir ? (
          <span className={`ft-chevron ${expanded ? "expanded" : ""}`}>
            <ChevronRight size={14} />
          </span>
        ) : (
          <span className="ft-chevron-placeholder" />
        )}
        {entry.is_dir ? (
          <Icon icon={"default" in folderIcon ? folderIcon.default : folderIcon} width={16} height={16} />
        ) : (
          <FileIcon name={entry.name} size={16} />
        )}
        <span>{entry.name}</span>
      </div>
      {entry.is_dir && children && (
        <div
          ref={childrenRef}
          className={`ft-children ${expanded ? "" : "collapsed"}`}
          style={{ maxHeight: expanded ? maxHeight : "0" }}
        >
          {children.map((child) => (
            <FileTreeNode
              key={child.path}
              entry={child}
              depth={depth + 1}
              expanded={expandedPaths.has(child.path)}
              active={child.path === activePath}
              children={childrenMap.get(child.path)}
              expandedPaths={expandedPaths}
              childrenMap={childrenMap}
              activePath={activePath}
              onToggle={onToggle}
              onSelect={onSelect}
            />
          ))}
          {children.length === 0 && expanded && (
            <div className="ft-empty" style={{ paddingLeft: (depth + 1) * 16 + 8 }}>—</div>
          )}
        </div>
      )}
    </div>
  );
}
```

### Étape 4 — Créer le composant FileTreePanel

- [ ] **Créer `src/components/file-tree/file-tree-panel.tsx`**

```tsx
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { filterTree } from "@/lib/file-tree-filter";
import { FileTreeFilter } from "./file-tree-filter";
import { FileTreeNode } from "./file-tree-node";
import type { useFileTree } from "@/hooks/use-file-tree";
import "./file-tree-panel.css";

interface FileTreePanelProps {
  tree: ReturnType<typeof useFileTree>;
  onFileSelect: (path: string) => void;
  activePath: string | null;
}

export function FileTreePanel({ tree, onFileSelect, activePath }: FileTreePanelProps) {
  const { t } = useTranslation();
  const [debouncedFilter, setDebouncedFilter] = useState("");
  const debounceRef = useMemo(() => ({ timer: 0 }), []);

  const handleFilterChange = (value: string) => {
    tree.setFilter(value);
    clearTimeout(debounceRef.timer);
    debounceRef.timer = window.setTimeout(() => setDebouncedFilter(value), 150);
  };

  const filtered = useMemo(
    () => filterTree(tree.rootEntries, tree.childrenMap, debouncedFilter),
    [tree.rootEntries, tree.childrenMap, debouncedFilter],
  );

  const mergedExpanded = useMemo(() => {
    const merged = new Set(tree.expandedPaths);
    for (const p of filtered.expandedPaths) merged.add(p);
    return merged;
  }, [tree.expandedPaths, filtered.expandedPaths]);

  return (
    <aside
      className={`ft-panel ${tree.open ? "open" : ""} ${tree.resizing ? "resizing" : ""}`}
      style={{ "--ft-width": `${tree.width}px` } as React.CSSProperties}
      aria-hidden={!tree.open}
    >
      <div className="ft-resize" onPointerDown={tree.startResize} />
      <div className="ft-head">
        <FileTreeFilter value={tree.filter} onChange={handleFilterChange} />
      </div>
      <div className="ft-body">
        {tree.loadError ? (
          <div className="ft-empty">{t("fileTree.loadError")}</div>
        ) : filtered.entries.length === 0 && debouncedFilter ? (
          <div className="ft-empty">{t("fileTree.noResults")}</div>
        ) : (
          filtered.entries.map((entry) => (
            <FileTreeNode
              key={entry.path}
              entry={entry}
              depth={0}
              expanded={mergedExpanded.has(entry.path)}
              active={entry.path === activePath}
              children={tree.childrenMap.get(entry.path)}
              expandedPaths={mergedExpanded}
              childrenMap={tree.childrenMap}
              activePath={activePath}
              onToggle={tree.toggleExpand}
              onSelect={onFileSelect}
            />
          ))
        )}
      </div>
    </aside>
  );
}
```

- [ ] **Vérifier le TS**

Run: `npx tsc --noEmit`
Expected: pas d'erreur

- [ ] **Commit**

```bash
git add src/components/file-tree/
git commit -m "feat(ui): add file tree components (panel, node, filter, CSS)"
```

---

## Task 7 : Intégration — bouton dossier + layout

**Files:**
- Modify: `src/components/file-preview/file-preview-panel.tsx`
- Modify: `src/components/agent-local/agent-chat-detail.tsx`
- Modify: `src/hooks/use-file-preview.ts` (ou le parent qui instancie les hooks)

### Étape 1 — Ajouter le bouton dossier dans le header du panel principal

- [ ] **Modifier `src/components/file-preview/file-preview-panel.tsx`**

Ajouter l'import :

```typescript
import { FolderTree } from "lucide-react";
```

Ajouter dans les props de `FilePreviewPanelProps` (après `onResizeStart`) :

```typescript
  hasProject?: boolean;
  treeOpen?: boolean;
  onToggleTree?: () => void;
```

Dans le JSX, ajouter le bouton **avant** le bouton fullscreen (avant `<button className="fp-icon-btn" onClick={() => props.onFullscreenChange(...)`) :

```tsx
        {props.hasProject && (
          <button
            className={`fp-icon-btn ${props.treeOpen ? "fp-icon-btn-active" : ""}`}
            onClick={props.onToggleTree}
            title={t("fileTree.toggleTree")}
          >
            <FolderTree size={16} />
          </button>
        )}
```

- [ ] **Ajouter le style actif dans `file-preview-panel.css`** (après `.fp-icon-btn:hover`) :

```css
.fp-icon-btn-active {
  color: var(--pulse);
  background: var(--surface-hover);
}
```

### Étape 2 — Intégrer FileTreePanel dans le layout

- [ ] **Modifier `src/components/agent-local/agent-chat-detail.tsx`**

Ajouter les imports :

```typescript
import { FileTreePanel } from "@/components/file-tree/file-tree-panel";
import type { useFileTree } from "@/hooks/use-file-tree";
```

Ajouter dans `AgentChatDetailProps` :

```typescript
  fileTree: ReturnType<typeof useFileTree>;
```

Dans le JSX, après `<FilePreviewPanel ... />` (ligne 88), ajouter :

```tsx
      <FileTreePanel
        tree={props.fileTree}
        onFileSelect={props.filePreview.openPath}
        activePath={props.filePreview.tabs.find((t) => t.id === props.filePreview.activeTab)?.path ?? null}
      />
```

Et dans `<FilePreviewPanel>`, passer les nouvelles props :

```tsx
        hasProject={props.fileTree.hasProject}
        treeOpen={props.fileTree.open}
        onToggleTree={props.fileTree.toggleOpen}
```

### Étape 3 — Instancier le hook use-file-tree dans le parent

- [ ] **Trouver où `useFilePreview` est instancié** et ajouter `useFileTree` au même endroit. C'est dans `src/components/agent-local/agent-local-tab.tsx`.

Ajouter l'import :

```typescript
import { useFileTree } from "@/hooks/use-file-tree";
```

Instancier le hook (après `useFilePreview`) :

```typescript
const fileTree = useFileTree(s.activeSessionId, activeProject?.path);
```

Passer à `AgentChatDetail` :

```tsx
fileTree={fileTree}
```

- [ ] **Vérifier le TS**

Run: `npx tsc --noEmit`
Expected: pas d'erreur

- [ ] **Commit**

```bash
git add src/components/file-preview/file-preview-panel.tsx src/components/file-preview/file-preview-panel.css src/components/agent-local/agent-chat-detail.tsx src/components/agent-local/agent-local-tab.tsx src/hooks/use-file-tree.ts
git commit -m "feat(ui): integrate file tree panel in layout with folder button"
```

---

## Task 8 : Test visuel + ajustements

**Files:** Aucun fichier spécifique — test manual end-to-end

### Étape 1 — Lancer le dev server

- [ ] **Lancer l'app**

Run: `npm run tauri dev`

### Étape 2 — Vérifier les cas

- [ ] **Session simple (pas de projet)** : le bouton dossier ne doit PAS être visible dans le header du panel principal
- [ ] **Session projet** : le bouton dossier est visible, clic → le tree panel slide-in depuis la droite
- [ ] **Navigation** : déplier des dossiers, vérifier l'animation du chevron et du max-height
- [ ] **Clic fichier** : ouvre la preview dans le panel principal (lecture seule, syntax highlighting)
- [ ] **Filtre** : taper dans la barre de recherche, vérifier le filtrage en temps réel sans flickering
- [ ] **Resize** : drag le bord gauche du tree, vérifier 240px min / 500px max
- [ ] **Fermeture** : re-clic sur le bouton dossier → tree slide-out vers la droite
- [ ] **Fullscreen** : panel principal en fullscreen + tree ouvert → les deux côte à côte, fermer le tree via le bouton → principal seul en fullscreen
- [ ] **Changement de session** : le tree se ferme et se reset
- [ ] **Thème dark + light** : vérifier que les couleurs sont correctes sur les deux thèmes
- [ ] **Fichiers masqués** : `.git` absent, `node_modules` présent, `.env` visible

### Étape 3 — Corriger les éventuels problèmes et committer

- [ ] **Commit final**

```bash
git add -A
git commit -m "fix: file tree panel adjustments after visual testing"
```

---

## Résumé des tâches

| # | Tâche | Fichiers | Tests |
|---|---|---|---|
| 1 | Backend FileEntry + list_directory | Rust: 3 créés, 3 modifiés | 10 tests Rust |
| 2 | Backend file watcher | Rust: 2 modifiés | — |
| 3 | Lib utilitaires (hidden + filter) | TS: 2 créés, 2 tests | 14 tests Vitest |
| 4 | i18n 7 langues | 7 JSON modifiés | — |
| 5 | Hook use-file-tree | TS: 1 créé | — |
| 6 | Composants file-tree | TSX: 3 créés, CSS: 1 créé | — |
| 7 | Intégration layout | TSX: 3 modifiés | — |
| 8 | Test visuel + ajustements | — | Manuel |
