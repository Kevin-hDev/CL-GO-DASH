# CL-GO-DASH

Desktop app agentique pour LLM locaux et cloud (style Claude Code). Tauri 2 + React 19, macOS/Linux/Windows.
Données dans `~/.local/share/cl-go-dash/` (macOS/Linux) ou `%APPDATA%\cl-go-dash` (Windows), centralisé via `services::paths::data_dir()`.

## Stack
- Tauri 2 (Rust) + React 19 + TypeScript + Vite
- Ollama embarqué en sidecar (`src-tauri/resources/ollama-bundle/`, téléchargé via `scripts/download-ollama.sh`)
- Vault chiffré XChaCha20-Poly1305 (master key dans le keyring OS, 1 seul accès au démarrage)
- Scheduler Tokio interne (pas de crontab)
- File watching via crate `notify` (FSEvents macOS, inotify Linux, ReadDirectoryChangesW Windows)

## Commands
- `npm run tauri dev` — dev mode
- `npm run tauri build` — release .app + .dmg
- `npx tsc --noEmit` — check TS
- `cd src-tauri && cargo check` — check Rust
- `cd src-tauri && cargo clippy --all-targets` — lint strict
- `cd src-tauri && cargo test` — tests unitaires

## Architecture

**Backend** (`src-tauri/src/`) :
- `commands/` — Tauri IPC, un fichier par domaine (heartbeat, agent_chat, agent_ollama, api_keys, llm, search, personality, projects, ...)
- `services/paths.rs` — chemin data centralisé cross-platform (`data_dir()`)
- `services/vault.rs` — vault chiffré XChaCha20-Poly1305 (encrypt/decrypt, master key keyring)
- `services/api_keys.rs` — gestion clés API en mémoire (`Zeroizing<String>`), persist via vault chiffré
- `services/agent_local/` — Ollama client, session store, outils, permission gate, types
- `services/llm/` — client unifié OpenAI-compat (Groq, Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek), catalog, streaming SSE
- `services/search/` — providers web search (Brave, Exa, Firecrawl) + routing `run_search`
- `services/stream_utils.rs` — `compute_tps`, `clean_think_tags` partagés entre Ollama et LLM
- `services/scheduler/` — scheduler Tokio (next_fire, fire_wakeup, log JSONL)
- `services/ollama_lifecycle.rs` — spawn/kill du sidecar Ollama (SIGTERM Unix, kill Windows)
- `services/config.rs` — lecture tolérante de `config.json`
- `services/terminal/` — PTY cross-platform (portable-pty)
- `tray.rs` — création tray icon (labels FR/EN dynamiques)
- `storage_migration.rs` — migration one-shot depuis CL-GO legacy
- `ollama_polling.rs` — polling status Ollama (événement `ollama-status`)
- `models/config.rs` — `ScheduledWakeup`, `WakeupSchedule` (enum `kind` once/daily/weekly)

**Frontend** (`src/`) :
- `components/{agent-local,heartbeat,ollama,personality,settings,layout,terminal,api-keys,ui}/`
- Chaque tab expose `{ list, detail }` via une tab function
- `hooks/` — logique extraite par domaine (sessions, chat, scroll, modèles, terminal, wakeups, etc.)
- `lib/platform.ts` — détection OS centralisée (`IS_MAC`, `MOD`, `ALT`, `MOD_KEY`)
- `i18n/` — traductions FR/EN (`en.json`, `fr.json`)

## Data sources (`~/.local/share/cl-go-dash/` ou `%APPDATA%\cl-go-dash`)

- `secrets.enc` — vault chiffré contenant les clés API
- `configured-providers.json` — registry des providers configurés
- `config.json` — `heartbeat.global_paused` + `scheduled_wakeups[]` + `advanced`
- `agent-sessions/*.json` — conversations Agent Local (flag `is_heartbeat` pour conversations auto-créées par réveils)
- `agent-settings.json` — mode permissions (auto/manuel)
- `agent-tabs.json` — état des onglets ouverts
- `memory/core/*.md` — fichiers personnalité
- `logs/wakeups.jsonl` — historique exécutions réveils (rolling 500 lignes)

## Rules

- **Paths** : TOUJOURS utiliser `crate::services::paths::data_dir()` — JAMAIS hardcoder le chemin. macOS/Linux = `~/.local/share/cl-go-dash`, Windows = `%APPDATA%\cl-go-dash`
- **Clés API** : vault chiffré XChaCha20 (`vault.rs`), master key unique dans le keyring OS (`keyring` crate). Les clés vivent en `Zeroizing<String>` en mémoire, zéroïsées après usage
- **JS ne voit JAMAIS une clé** : les commandes Tauri exposent `set_api_key / delete_api_key / has_api_key / list_configured_providers / test_api_key` — aucune n'expose `get`. Rust charge la clé au moment de l'appel HTTPS et la zéroïse après
- **Collections bornées** : `ActiveStreams` max 32, PTY sessions max 16, messages par session max 2000
- **Path traversal** : toujours `canonicalize()` + `starts_with()` avant de lire/écrire un fichier dont le chemin vient du frontend
- **Logs** : ne jamais logger de body HTTP brut des providers — utiliser `sanitize_log_body()` (tronque à 200 chars)
- `envPrefix: ["VITE_"]` uniquement dans `vite.config.ts` (CVE-2023-46115 : ne jamais ajouter `"TAURI_"`)
- Écriture config : atomique (tmp + rename)
- Après toute mutation wakeup : appeler `scheduler.notify_config_changed()`
- Fichiers code/test < 200 lignes (découper si dépassement)
- String slicing UTF-8 safe : `char_indices()`, jamais byte slice
- File watcher debounce 200ms
- Ollama téléchargé via `cd src-tauri && bash scripts/download-ollama.sh` (pas de Git LFS)

## i18n
- Tu gères les traductions Français/anglais quand tu ajoutes du texte dans l'application, tu ne codes aucun texte en dur
- Messages d'erreur visibles : toujours via clés i18n, jamais `String(err)` brut
- Hooks non-React : importer `i18n` depuis `@/i18n` directement (pas de `useTranslation`)
