# Cross-plateforme — Installation, Build, Mise à jour

## Architecture du système de mise à jour

### Détection des mises à jour (identique sur les 3 OS)

- **Backend Rust** : `commands/app_update.rs`
  - `check_app_update()` — appelle `api.github.com/repos/Kevin-hDev/CL-GO-DASH/releases/latest`
  - Compare la version locale (`env!("CARGO_PKG_VERSION")`) avec le tag de la release
  - Cherche l'asset correspondant à l'OS : `.dmg` (macOS), `.AppImage` (Linux), `-setup.exe` (Windows NSIS)
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
| **Windows** | `-setup.exe` (NSIS) | Télécharge dans `%TEMP%` → lance l'installeur avec `/S` (silencieux) |

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

```powershell
irm https://raw.githubusercontent.com/Kevin-hDev/CL-GO-DASH/main/install.ps1 | iex
```

Le script `install.ps1` cherche d'abord `.msi` puis `.exe` dans les assets de la release. Il lance `msiexec /i /passive` pour un MSI ou l'exe avec `/S` pour un NSIS.

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
- `windows-latest` → x64 → NSIS installer (`.exe`)

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
- Rien à installer. L'installeur NSIS gère tout.

---

## Accélération matérielle (GPU)

### État des lieux (avril 2026)

Le bundle Ollama (`ollama-windows-amd64.zip` / `ollama-linux-amd64.tar.zst`) inclut les libs GPU dans `lib/ollama/`. Ollama est téléchargé au premier lancement dans `~/.local/share/cl-go-dash/ollama-bundle/`.

### Par OS et GPU

| OS | GPU | Backend | Comment ça marche |
|---|---|---|---|
| **macOS** | Apple Silicon | Metal | Intégré dans le binaire `ollama-darwin.tgz`, rien à configurer |
| **Linux** | NVIDIA | CUDA | Le bundle générique inclut `lib/ollama/libggml-cuda.so` — seul le driver NVIDIA ≥531 est requis côté système |
| **Linux** | AMD | ROCm | Le bundle `ollama-linux-amd64-rocm.tar.zst` est sélectionné automatiquement par `select_archive_name()` quand un GPU AMD est détecté |
| **Windows** | NVIDIA | CUDA | Le bundle inclut `lib/ollama/cuda_v12/` et `cuda_v13/` — Ollama auto-détecte CUDA, rien à configurer |
| **Windows** | AMD | Vulkan | `OLLAMA_VULKAN=1` injecté automatiquement par `ollama_lifecycle.rs` quand un GPU AMD est détecté |

### Détection GPU (`services/gpu_detect.rs`)

- **Linux** : lit les vendor IDs dans `/sys/class/drm/*/device/vendor` (PCI : `0x10de` = NVIDIA, `0x1002` = AMD, `0x8086` = Intel)
- **Windows** : PowerShell `Get-CimInstance Win32_VideoController`, match sur le nom ("nvidia", "geforce", "amd", "radeon", etc.)
- **macOS** : retourne `Unknown` — Metal est toujours disponible via le binaire macOS

### Sélecteur CPU/GPU (`Settings > Avancé`)

L'utilisateur peut choisir entre CPU et GPU dans les paramètres avancés (masqué sur macOS). Le choix est stocké dans `config.json` → `advanced.hardware_accel` (`"cpu"` ou `"gpu"`, défaut `"gpu"`).

Au démarrage du sidecar (`ollama_lifecycle.rs`) :
- `"cpu"` → injecte `OLLAMA_LLM_LIBRARY=cpu` (force le CPU)
- `"gpu"` → injecte `OLLAMA_VULKAN=1` si AMD sur Windows, sinon laisse Ollama auto-détecter

Un bouton "Restart Ollama" apparaît dans les settings après changement pour appliquer immédiatement.

### Pourquoi Vulkan et pas ROCm sur Windows ?

**ROCm Windows** (7.2.2, avril 2026) n'est pas encore production-ready pour Ollama :
- Auto-détection GPU instable (issues ollama #14686, #12573, #15715)
- Les RX 9000 nécessitent des hacks manuels (`ROCBLAS_TENSILE_LIBPATH`)
- Le zip ROCm (`ollama-windows-amd64-rocm.zip`) est un complément, pas un standalone

**Vulkan** est déjà bundlé (`lib/ollama/vulkan/ggml-vulkan.dll`), couvre tous les GPU AMD (RX 5000 à 9000) et fonctionne sans dépendance externe. Seule limitation : Ollama utilise un llama.cpp daté (b7437, déc. 2025) avec ~56% de perf en moins que le potentiel (issue ollama #15601). Ça se comblera quand Ollama mettra à jour.

**Gemma 4 + Vulkan** : le modèle produit des réponses incohérentes (LaTeX, boucles) car son architecture (KV cache partagée) n'est pas correctement gérée par le vieux backend Vulkan d'Ollama (issues ollama #15261, llama.cpp #21516). Workaround : passer en CPU via le sélecteur.

### Logs sidecar

Les logs stderr d'Ollama sont écrits dans `~/.local/share/cl-go-dash/logs/ollama-sidecar.log` (écrasé à chaque démarrage). Utile pour diagnostiquer les problèmes GPU.

---

## Corrections Windows (v0.6.8 → v0.7.1)

| Version | Problème | Cause | Fix |
|---|---|---|---|
| v0.6.8 | Ollama sur CPU malgré GPU AMD | Aucune variable GPU injectée au sidecar | Injection `OLLAMA_VULKAN=1` + logs stderr |
| v0.6.9 | Bouton mise à jour invisible | Code cherchait `.msi`, le CI produit NSIS `.exe` | `platform_extension()` → `-setup.exe` + lancement direct NSIS `/S` |
| v0.7.0 | Toggles personnalité cassés | `path.split("/")` ne sépare pas les backslash Windows | `path.split(/[\\/]/)` |
| v0.7.0 | Pas de choix CPU/GPU | — | Sélecteur dans Settings/Avancé + `restart_ollama_sidecar` |
| v0.7.1 | Contour fenêtre invisible | Padding 1px insuffisant sur Windows/Linux | Padding 3px sur `.os-other` uniquement |

---

## Fichiers concernés

```
install.sh                                  # Script première installation macOS/Linux
install.ps1                                 # Script première installation Windows
.github/workflows/release.yml              # CI build cross-plateforme
src-tauri/src/commands/app_update.rs        # Check version GitHub Releases
src-tauri/src/commands/app_update_install.rs # Download + auto-install par OS
src-tauri/src/commands/ollama_updates.rs    # Check mises à jour modèles Ollama
src-tauri/src/commands/ollama_setup.rs      # Download Ollama + restart sidecar
src-tauri/src/services/ollama_lifecycle.rs  # Spawn/kill sidecar + injection env GPU
src-tauri/src/services/gpu_detect.rs        # Détection GPU (sysfs Linux, PowerShell Windows)
src-tauri/src/models/config.rs              # AdvancedSettings.hardware_accel
src/hooks/use-update-checker.ts             # Hook React — check toutes les heures
src/components/layout/update-notifications.tsx  # UI bulles de notification
src/components/layout/update-notifications.css  # Styles des bulles
src/components/layout/window-toolbar.tsx    # Icône update dans le toolbar
src/components/settings/advanced-settings.tsx   # Sélecteur CPU/GPU
src/components/settings/hardware-accel-control.tsx # Composant bouton restart
src/components/personality/personality-tab.tsx   # Toggles injection contexte
```

---

## Points d'attention

- **Repo public** : obligatoire pour que l'API GitHub Releases fonctionne sans token d'authentification
- **Pas de code signing** : macOS Gatekeeper bloque si téléchargé via navigateur → d'où le `curl` (pas de quarantaine) et le téléchargement direct depuis l'app en Rust
- **Ollama téléchargé au premier lancement** : le CI ne bundle PAS Ollama — il est téléchargé dans `~/.local/share/cl-go-dash/ollama-bundle/` au premier lancement
- **Ubuntu 22.04** : builder sur la plus vieille version cible garantit la compatibilité avec 22.04, 24.04, 25.04 (glibc forward-compatible)
- **Chemins Windows** : toujours utiliser `split(/[\\/]/)` côté frontend pour extraire un nom de fichier d'un path — `split("/")` ne fonctionne pas avec les backslash Windows
- **ROCm Linux** : production-ready (v7.2.2), auto-détecté par `select_archive_name()`. Le bundle ROCm est téléchargé automatiquement pour les GPU AMD
- **ROCm Windows** : pas fiable pour Ollama (avril 2026), Vulkan utilisé à la place
- **Windows Defender — Accès contrôlé aux dossiers** : au premier lancement, `ollama.exe` (binaire non signé) peut être bloqué par la protection anti-ransomware quand il essaie d'écrire sur le disque (modèles dans `~/.ollama/models/`). L'utilisateur doit cliquer "Autoriser" dans la notification — ça ne redemande plus ensuite. Pas de contournement possible sans code signing
