# CL-GO-DASH

Application desktop (Tauri 2 + React 19) pour discuter avec des LLM locaux via Ollama et planifier des réveils automatisés qui envoient des prompts à intervalles réguliers.

## Fonctionnalités

- **Agent Local** : chat avec n'importe quel modèle Ollama installé ou via API distant (Groq, Gemini, Mistral, OpenAI, OpenRouter…), gestion des conversations en onglets, permissions manuelles/auto sur les outils (bash, write_file, web_fetch)
- **Réveils** : scheduler interne qui prompt un LLM à heure fixe (ponctuel / journalier / hebdomadaire), réponses stockées dans une conversation dédiée par modèle. Supporte Ollama local + providers API tool-capable.
- **Clés API** : gestion centralisée des providers LLM et search (Brave, Exa, Firecrawl) via un onglet dédié. Clés stockées dans le **keystore OS natif** (macOS Keychain, Windows DPAPI, Linux Secret Service) — jamais en clair sur disque.
- **Ollama embarqué** : plus besoin d'installer Ollama.app séparément — le binaire est bundlé dans l'application
- **Personnalité** : édition des fichiers Markdown dans `~/.local/share/cl-go-dash/memory/core/`
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
- **File watching** : crate `notify` (FSEvents sur macOS)

## Prérequis

- macOS (Apple Silicon M1/M2/M3/M4/M5)
- Node.js 20+
- Rust (via `rustup`)
- **Git LFS** : requis pour cloner le repo (le binaire Ollama bundlé y est stocké)

## Installation

```bash
# 1. Installer Git LFS (une fois par machine)
brew install git-lfs
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
npm run tauri build     # Build release (.app + .dmg)
npx tsc --noEmit        # Check TypeScript
cd src-tauri && cargo check    # Check Rust
cd src-tauri && cargo test     # Tests unitaires
```

## Architecture

```
src-tauri/                # Backend Rust + Tauri
├── src/
│   ├── commands/         # Tauri IPC commands (agent, heartbeat, personality, ...)
│   ├── services/         # Logique métier
│   │   ├── agent_local/  # Ollama client, session store, outils
│   │   ├── scheduler/    # Scheduler Tokio interne (réveils)
│   │   ├── config.rs     # Lecture/écriture config.json tolérante
│   │   ├── file_watcher.rs  # Watch FS pour auto-refresh
│   │   └── ollama_lifecycle.rs  # Gestion du sidecar Ollama
│   └── models/config.rs  # Schémas ScheduledWakeup, HeartbeatConfig
└── resources/ollama-bundle/  # Binaire Ollama + libs (via Git LFS)

src/                      # Frontend React
├── components/
│   ├── agent-local/      # Chat, permissions, tabs
│   ├── heartbeat/        # Grid réveils, popup création, détails
│   ├── ollama/           # Modelfile editor, model browser
│   ├── personality/      # Markdown viewer
│   └── layout/           # Sidebar, drag region
├── hooks/                # useWakeups, useAgentChat, useAgentSessions, ...
├── types/                # Types TS alignés sur Rust
└── i18n/                 # FR + EN
```

## Stockage local

Toutes les données utilisateur sont dans `~/.local/share/cl-go-dash/` :

| Chemin | Contenu |
|---|---|
| `agent-sessions/*.json` | Conversations Agent Local |
| `agent-settings.json` | Mode permissions (auto/manuel) |
| `agent-tabs.json` | État des onglets ouverts |
| `config.json` | Heartbeat config + scheduled_wakeups |
| `memory/core/*.md` | Fichiers de personnalité |
| `logs/wakeups.jsonl` | Historique d'exécution des réveils (rolling 500 lignes) |

## Ollama — sidecar embarqué

L'application embarque **Ollama** comme sidecar pour éviter toute dépendance externe :

- Au démarrage, l'app vérifie si un daemon Ollama tourne déjà sur `localhost:11434`
- Si oui (Ollama.app déjà installée), elle l'utilise tel quel
- Si non, elle lance son propre binaire bundlé
- À la fermeture, le sidecar est arrêté proprement (SIGTERM + grace period 3s)

**Les modèles sont partagés** avec Ollama.app si elle est installée (`~/.ollama/models/`).

## Règles internes

- **Stockage des secrets** (clés API à venir) : keystore OS natif via `tauri-plugin-secure-storage` — jamais dans des fichiers config en clair
- **Écriture atomique** : tous les fichiers de config sont écrits en `.tmp` + rename
- **Fichiers < 200 lignes** : tout fichier code est découpé si nécessaire
- **Pas de crontab système** : le scheduler est interne à l'app (Tokio), nécessite l'app ouverte pour que les réveils se déclenchent

## Licence

Privé — projet personnel.
