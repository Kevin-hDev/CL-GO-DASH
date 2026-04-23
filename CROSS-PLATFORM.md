# Cross-plateforme — Installation, Build, Mise à jour

## Architecture du système de mise à jour

### Détection des mises à jour (identique sur les 3 OS)

- **Backend Rust** : `commands/app_update.rs`
  - `check_app_update()` — appelle `api.github.com/repos/Kevin-hDev/CL-GO-DASH/releases/latest`
  - Compare la version locale (`env!("CARGO_PKG_VERSION")`) avec le tag de la release
  - Cherche l'asset correspondant à l'OS : `.dmg` (macOS), `.AppImage` (Linux), `.msi` (Windows)
  - Retourne `{ version, assetUrl }` ou `None`

- **Backend Rust** : `commands/check_ollama_updates`
  - Liste les modèles installés, fetch les tags registry pour chaque famille
  - Compare les digests SHA256 — retourne la liste des modèles avec update dispo

- **Frontend** : `hooks/use-update-checker.ts`
  - Check au lancement + toutes les heures
  - Écoute aussi `ollama-models-changed` pour re-vérifier après un pull

- **Frontend** : `components/layout/update-notifications.tsx`
  - Bulles style macOS, animation slide-down 500ms staggerée
  - Barre de progression orange pour le téléchargement/pull

### Téléchargement + auto-install (par OS)

**Backend Rust** : `commands/app_update_install.rs`

| OS | Asset | Processus |
|---|---|---|
| **macOS** | `.dmg` | Télécharge dans `/tmp/` → `hdiutil attach` → `cp -Rf` dans `/Applications/` → `hdiutil detach` → `open` la nouvelle app |
| **Linux** | `.AppImage` | Télécharge dans `/tmp/` → `cp` dans `~/.local/bin/CL-GO.AppImage` → `chmod +x` → lance |
| **Windows** | `.msi` | Télécharge dans `%TEMP%` → `msiexec /i /passive` (install silencieuse) |

L'app se ferme automatiquement (`app.exit(0)`) après avoir lancé un script shell (macOS/Linux) ou l'installeur (Windows) en arrière-plan. Le script attend que l'app soit fermée avant de copier.

---

## Première installation

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Kevin-hDev/CL-GO-DASH/main/install.sh | bash
```

Le script `install.sh` :
1. Détecte l'OS (`uname -s`) et l'architecture (`uname -m`)
2. Sur Linux : vérifie et installe les dépendances (`libwebkit2gtk-4.1-0`, `libgtk-3-0`) via `apt-get` ou `dnf`
3. Appelle l'API GitHub Releases pour trouver la dernière version
4. Télécharge l'asset correspondant (`.dmg` ou `.AppImage`) via `curl` (pas de quarantaine macOS)
5. Installe :
   - macOS : mount DMG → copie dans `/Applications/` → démonte → lance
   - Linux : copie dans `~/.local/bin/` → chmod +x → lance

### Windows

Télécharger le `.msi` depuis la page GitHub Releases et double-cliquer. SmartScreen peut afficher un avertissement (pas de code signing) mais laisse passer.

---

## Build et release

### Versions — 3 fichiers à synchroniser

| Fichier | Champ |
|---|---|
| `src-tauri/tauri.conf.json` | `"version": "X.Y.Z"` |
| `src-tauri/Cargo.toml` | `version = "X.Y.Z"` |
| `package.json` | `"version": "X.Y.Z"` |

### CI GitHub Actions

**Workflow** : `.github/workflows/release.yml`

Se déclenche automatiquement quand on push un tag `v*`, ou manuellement via GitHub Actions.

**Builds** :
- `macos-latest` → ARM64 (Apple Silicon) → `.dmg`
- `ubuntu-22.04` → x64 → `.AppImage` + `.deb`
- `windows-latest` → x64 → `.msi`

**Processus pour publier une release** :

```bash
# 1. Bumper la version dans les 3 fichiers
# 2. Commit + push
git add src-tauri/tauri.conf.json src-tauri/Cargo.toml package.json
git commit -m "chore: bump version to X.Y.Z"
git push

# 3. Créer et pusher le tag → déclenche le CI automatiquement
git tag vX.Y.Z
git push origin vX.Y.Z
```

Le CI crée une **release draft** avec tous les assets. Il reste à la publier sur GitHub.

### Build local (macOS uniquement)

```bash
npm run tauri build
# → src-tauri/target/release/bundle/macos/CL-GO.app
# → src-tauri/target/release/bundle/dmg/CL-GO_X.Y.Z_aarch64.dmg
```

---

## Dépendances par OS

### macOS
- Rien à installer côté utilisateur. `curl` et `hdiutil` sont pré-installés.

### Linux (Ubuntu/Debian)
- `libwebkit2gtk-4.1-0` — moteur web pour Tauri 2
- `libgtk-3-0` — toolkit graphique
- Installées automatiquement par `install.sh` via `apt-get`

### Linux (Fedora/RHEL)
- `webkit2gtk4.1`
- `gtk3`
- Installées automatiquement par `install.sh` via `dnf`

### Windows
- Rien à installer. Le `.msi` gère tout.

---

## Fichiers concernés

```
install.sh                                  # Script première installation macOS/Linux
.github/workflows/release.yml              # CI build cross-plateforme
src-tauri/src/commands/app_update.rs        # Check version GitHub Releases
src-tauri/src/commands/app_update_install.rs # Download + auto-install par OS
src-tauri/src/commands/ollama_updates.rs    # Check mises à jour modèles Ollama
src/hooks/use-update-checker.ts             # Hook React — check toutes les heures
src/components/layout/update-notifications.tsx  # UI bulles de notification
src/components/layout/update-notifications.css  # Styles des bulles
src/components/layout/window-toolbar.tsx    # Icône update dans le toolbar
```

---

## Points d'attention

- **Repo public** : obligatoire pour que l'API GitHub Releases fonctionne sans token d'authentification
- **Pas de code signing** : macOS Gatekeeper bloque si téléchargé via navigateur → d'où le `curl` (pas de quarantaine) et le téléchargement direct depuis l'app en Rust
- **Git LFS** : le bundle Ollama (~380 MB) est en LFS → le CI doit cloner avec `lfs: true`
- **Ubuntu 22.04** : builder sur la plus vieille version cible garantit la compatibilité avec 22.04, 24.04, 25.04 (glibc forward-compatible)
