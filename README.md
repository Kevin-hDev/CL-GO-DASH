# CL-GO-DASH

Application desktop agentique (Tauri 2 + React 19) pour LLM locaux via Ollama et providers cloud (Groq, Gemini, Mistral, OpenAI, OpenRouter, DeepSeek, Cerebras). Chat en onglets, outils (bash, fichiers, web), réveils automatisés, terminal intégré.

## Fonctionnalités

- **Agent Local** : chat avec n'importe quel modèle Ollama ou provider cloud, gestion des conversations en onglets, permissions manuelles/auto sur les outils (bash, write_file, web_fetch), réflexion approfondie (thinking)
- **Réveils** : scheduler interne qui prompt un LLM à heure fixe (ponctuel / journalier / hebdomadaire), réponses stockées dans une conversation dédiée par modèle
- **Clés API** : gestion centralisée des providers LLM et search. Clés stockées dans un **vault chiffré XChaCha20-Poly1305** (master key dans le keyring OS) — jamais en clair sur disque, jamais exposées au frontend
- **Ollama embarqué** : plus besoin d'installer Ollama.app séparément — le binaire est bundlé dans l'application
- **Terminal intégré** : PTY cross-platform avec onglets, raccourci Cmd/Ctrl+J
- **Personnalité** : édition des fichiers Markdown de contexte
- **Ollama browser** : recherche de modèles, pull, édition de modelfiles

## Providers supportés (free-tier friendly)

| Type | Provider | Free tier |
|---|---|---|
| LLM | [Groq](https://console.groq.com/keys) | 14 400 req/day |
| LLM | [Google Gemini](https://aistudio.google.com/app/apikey) | 1M tokens/min |
| LLM | [Mistral](https://console.mistral.ai/api-keys) | 1B tokens/month |
| LLM | [Cerebras](https://cloud.cerebras.ai/) | 1M tokens/day |
| LLM | [OpenRouter](https://openrouter.ai/settings/keys) | 30+ free models |
| LLM | [OpenAI](https://platform.openai.com/api-keys) | $5 signup credits |
| LLM | [DeepSeek](https://platform.deepseek.com/api_keys) | Low-cost ($0.30/Mtok) |
| Search | [Brave Search](https://api-dashboard.search.brave.com/app/keys) | 2 000 req/month |
| Search | [Exa](https://dashboard.exa.ai/api-keys) | 1 000 req/month |
| Scraping | [Firecrawl](https://www.firecrawl.dev/app/api-keys) | 500 crédits |

## Stack technique

- **Backend** : Rust + Tauri 2
- **Frontend** : React 19 + TypeScript + Vite
- **LLM runtime** : Ollama (embarqué comme sidecar)
- **Sécurité** : vault XChaCha20-Poly1305, master key dans keyring OS (macOS Keychain / Windows DPAPI / Linux Secret Service)
- **File watching** : crate `notify` (FSEvents macOS, inotify Linux, ReadDirectoryChangesW Windows)

## Prérequis

- macOS (Apple Silicon ou Intel), Linux, ou Windows
- Node.js 20+
- Rust (via `rustup`)
- **Git LFS** : requis pour cloner le repo (le binaire Ollama bundlé y est stocké)

## Installation

### macOS / Linux (une commande)

```bash
curl -fsSL https://raw.githubusercontent.com/Kevin-hDev/CL-GO-DASH/main/install.sh | bash
```

Télécharge la dernière release, installe l'app et la lance automatiquement.
- **macOS** : installe dans `/Applications/`
- **Linux** : installe dans `~/.local/bin/` (dépendances système installées automatiquement)

### Windows

Télécharge le `.msi` depuis la [dernière release](https://github.com/Kevin-hDev/CL-GO-DASH/releases/latest) et lance l'installateur.

### Mises à jour

Les mises à jour sont automatiques : une notification apparaît dans l'app quand une nouvelle version est disponible. Un clic et l'app se met à jour toute seule.

---

## Développement

### Prérequis
- **Git LFS** : requis pour cloner le repo (le binaire Ollama bundlé y est stocké)

```bash
# 1. Installer Git LFS (une fois par machine)
# macOS :
brew install git-lfs
# Linux :
sudo apt install git-lfs   # ou equiv. pour votre distro
# Windows :
winget install GitHub.GitLFS

git lfs install

# 2. Cloner le repo
git clone https://github.com/Kevin-hDev/CL-GO-DASH.git
cd CL-GO-DASH

# 3. Installer les dépendances
npm install
```

> **Sans Git LFS**, le clone récupérera seulement les pointeurs texte des gros fichiers (binaire Ollama, libs Metal). L'app ne pourra pas démarrer le sidecar.

## Commandes

```bash
npm run tauri dev       # Mode dev (hot reload)
npm run tauri build     # Build release (.app + .dmg / .msi / .deb)
npx tsc --noEmit        # Check TypeScript
cd src-tauri && cargo check    # Check Rust
cd src-tauri && cargo clippy --all-targets  # Lint strict
cd src-tauri && cargo test     # Tests unitaires
```

## Architecture

```
src-tauri/                # Backend Rust + Tauri
├── src/
│   ├── commands/         # Tauri IPC (agent_chat, heartbeat, personality, api_keys, llm, ...)
│   ├── services/
│   │   ├── agent_local/  # Ollama client, session store, outils, permission gate
│   │   ├── llm/          # Client unifié OpenAI-compat, catalog, streaming SSE
│   │   ├── search/       # Brave, Exa, Firecrawl + routing
│   │   ├── scheduler/    # Scheduler Tokio interne (réveils)
│   │   ├── terminal/     # PTY cross-platform (portable-pty)
│   │   ├── paths.rs      # Chemin data centralisé cross-platform
│   │   ├── vault.rs      # Vault chiffré XChaCha20-Poly1305
│   │   ├── api_keys.rs   # Gestion clés API (Zeroizing en mémoire)
│   │   ├── config.rs     # Lecture/écriture config.json tolérante
│   │   ├── stream_utils.rs  # compute_tps, clean_think_tags partagés
│   │   └── ollama_lifecycle.rs  # Gestion du sidecar Ollama
│   ├── tray.rs           # Tray icon (labels FR/EN dynamiques)
│   ├── storage_migration.rs  # Migration one-shot depuis CL-GO legacy
│   ├── ollama_polling.rs # Polling status Ollama
│   └── models/           # Schémas ScheduledWakeup, HeartbeatConfig
└── resources/ollama-bundle/  # Binaire Ollama + libs (via Git LFS)

src/                      # Frontend React
├── components/
│   ├── agent-local/      # Chat, permissions, tabs
│   ├── heartbeat/        # Grid réveils, popup création, détails
│   ├── ollama/           # Modelfile editor, model browser
│   ├── personality/      # Markdown viewer
│   ├── settings/         # Général, raccourcis, avancé, about
│   ├── terminal/         # Terminal intégré PTY
│   ├── api-keys/         # Configuration clés API
│   ├── layout/           # Sidebar, toolbar, drag region
│   └── ui/               # Primitives réutilisables
├── hooks/                # Logique extraite par domaine
├── lib/                  # platform.ts (détection OS)
├── types/                # Types TS alignés sur Rust
└── i18n/                 # FR + EN
```

## Stockage local

Données dans `~/.local/share/cl-go-dash/` (macOS/Linux) ou `%APPDATA%\cl-go-dash` (Windows) :

| Chemin | Contenu |
|---|---|
| `secrets.enc` | Vault chiffré contenant les clés API |
| `configured-providers.json` | Registry des providers configurés |
| `config.json` | Heartbeat config + scheduled_wakeups + advanced settings |
| `agent-sessions/*.json` | Conversations Agent Local |
| `agent-settings.json` | Mode permissions (auto/manuel) |
| `agent-tabs.json` | État des onglets ouverts |
| `memory/core/*.md` | Fichiers de personnalité |
| `logs/wakeups.jsonl` | Historique d'exécution des réveils (rolling 500 lignes) |

## Ollama — sidecar embarqué

L'application embarque **Ollama** comme sidecar pour éviter toute dépendance externe :

- Au démarrage, l'app vérifie si un daemon Ollama tourne déjà sur `localhost:11434`
- Si oui (Ollama.app déjà installée), elle l'utilise tel quel
- Si non, elle lance son propre binaire bundlé
- À la fermeture, le sidecar est arrêté proprement (SIGTERM Unix / kill Windows + grace period 3s)

**Les modèles sont partagés** avec Ollama.app si elle est installée (`~/.ollama/models/`).

## Sécurité

- **Vault chiffré** : clés API chiffrées XChaCha20-Poly1305, master key dans le keyring OS natif
- **Zéroïsation** : tous les secrets en mémoire dans `Zeroizing<String>`, buffers intermédiaires zéroïsés après usage
- **JS ne voit jamais une clé** : aucune commande Tauri n'expose `get_api_key`
- **Path traversal** : canonicalize + starts_with sur tout chemin venant du frontend
- **Collections bornées** : ActiveStreams (32), PTY sessions (16), messages par session (2000)
- **Logs filtrés** : body HTTP providers tronqué à 200 chars, jamais de secret dans les logs

## Licence

Privé — projet personnel.
