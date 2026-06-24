# CL-GO-DASH

Agentic desktop application (Tauri 2 + React 19) for local LLMs via Ollama and cloud providers (Groq, Gemini, Mistral, OpenAI, OpenRouter, DeepSeek, Cerebras, xAI, Moonshot Kimi, Z.ai GLM). Tabbed chat, tools, subagents, MCP connectors, automated wakeups, forecasting, Git workflows, file previews, and an integrated terminal.

## Features

- **Local Agent**: chat with any Ollama model or cloud provider, tabbed conversation management, manual/auto/chat permissions for tools, advanced reasoning, model favorites, projects, and `AGENTS.md` context
- **Tools**: bash, file read/write, web fetch/search, Git actions, file tree, file preview, Office preview, link preview, MCP tools, and forecasting tools
- **Subagents**: launch isolated assistant runs from a conversation and merge their results back into the main chat
- **Wakeups**: internal scheduler that prompts an LLM at a fixed time (one-time / daily / weekly), with responses stored in a dedicated conversation per model
- **Forecast**: time-series forecasting with local and cloud models, history, comparisons, scenarios, notes, exports, and agent-callable analysis
- **MCP connectors**: cloud and local connectors with OAuth or environment tokens, status testing, and per-chat activation
- **Gateway / channels**: optional background gateway for external channels such as Telegram, Slack, and Discord
- **API keys**: centralized management for LLM, search, forecast, MCP, and gateway credentials. Keys are stored in an **XChaCha20-Poly1305 encrypted vault** (master key in the OS keyring) — never plaintext on disk, never exposed to the frontend
- **Bundled Ollama**: downloaded on first launch, no need to install Ollama separately
- **Git branch management**: branch selector in chat with switching, inline creation, worktree navigation, real-time file watcher, conflict dialog with automatic WIP commit
- **Integrated terminal**: cross-platform PTY with tabs, Cmd/Ctrl+J shortcut
- **Personality and memory**: edit Markdown context files, personality injection, and local memory folders
- **Ollama browser**: model search, pull, modelfile editing

## Supported providers

| Type | Provider | Limit / pricing label |
|---|---|---|
| LLM | [Groq](https://console.groq.com/keys) | 14,400 req/day |
| LLM | [Google Gemini](https://aistudio.google.com/app/apikey) | 1M tokens/min |
| LLM | [Mistral](https://console.mistral.ai/api-keys) | 1B tokens/month |
| LLM | [Cerebras](https://cloud.cerebras.ai/) | 1M tokens/day |
| LLM | [OpenRouter](https://openrouter.ai/settings/keys) | 30+ free models |
| LLM | [OpenAI](https://platform.openai.com/api-keys) | $5 signup credits |
| LLM | [DeepSeek](https://platform.deepseek.com/api_keys) | Low-cost ($0.14/Mtok) |
| LLM | [xAI](https://console.x.ai) | Budget ($0.20/Mtok) |
| LLM | [Moonshot Kimi](https://platform.kimi.ai/console/api-keys) | Low-cost ($0.60/Mtok) |
| LLM | [Z.ai GLM](https://z.ai/manage-apikey/apikey-list) | GLM-4.5-Flash free |
| Search | [Brave Search](https://api-dashboard.search.brave.com/app/keys) | 2,000 req/month |
| Search | [Exa](https://dashboard.exa.ai/api-keys) | 1,000 req/month |
| Scraping | [Firecrawl](https://www.firecrawl.dev/app/api-keys) | 500 credits |
| Forecast | [Nixtla TimeGPT](https://dashboard.nixtla.io/) | Cloud forecast API |

## Forecast models

CL-GO-DASH includes a Forecast workspace for time-series analysis:

- **Local families**: Amazon Chronos / Chronos-Bolt, Google TimesFM, Datadog Toto 2.0, Salesforce MOIRAI 2.0, IBM FlowState, PriorLabs TabPFN-TS, NX-AI TiRex, Kairos, and THUML Sundial
- **Cloud family**: Nixtla TimeGPT-2 / TimeGPT-2.1
- **Workflow**: run forecasts, keep analysis history, compare runs, create scenarios, add notes, export results, and call forecast tools from the Local Agent

## Technical stack

- **Backend**: Rust + Tauri 2
- **Frontend**: React 19 + TypeScript + Vite
- **LLM runtime**: Ollama (bundled as a sidecar)
- **Forecast runtime**: local forecast sidecar plus optional Nixtla API
- **Connector runtime**: MCP bridge, OAuth storage, and Gateway channel service
- **Security**: XChaCha20-Poly1305 vault, master key in the OS keyring (macOS Keychain / Windows DPAPI / Linux Secret Service)
- **File watching**: `notify` crate (FSEvents on macOS, inotify on Linux, ReadDirectoryChangesW on Windows)

## Prerequisites

- macOS (Apple Silicon), Linux, or Windows
- Node.js 20+
- Rust (via `rustup`)

## Installation

### macOS / Linux (one command)

```bash
curl -fsSL https://raw.githubusercontent.com/Kevin-hDev/CL-GO-DASH/main/install.sh | bash
```

Downloads the latest release, installs the app, and launches it automatically.
- **macOS**: installs into `/Applications/`
- **Linux**: installs the Debian package through `apt-get` (Ubuntu/Debian only)

The Linux installer uses the `.deb` release asset so the app is visible in the system application menu.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Kevin-hDev/CL-GO-DASH/main/install.ps1 | iex
```

Downloads the latest release and launches the Windows NSIS `-setup.exe` installer automatically.

> **Windows Defender**: on first launch, "Controlled folder access" may block `ollama.exe`. Click "Allow" in the notification — it will not ask again.

### Updates

Updates are automatic: a notification appears in the app when a new version is available. One click and the app updates itself.

---

## Development

```bash
# 1. Clone the repo
git clone https://github.com/Kevin-hDev/CL-GO-DASH.git
cd CL-GO-DASH

# 2. Install dependencies
npm install

# 3. Download the Ollama binary for your OS
cd src-tauri && bash scripts/download-ollama.sh
```

## Commands

```bash
npm run tauri dev       # Dev mode (hot reload)
npm run tauri build     # Release build (.dmg / -setup.exe / .deb)
npx tsc --noEmit        # TypeScript check
cd src-tauri && cargo check    # Rust check
cd src-tauri && cargo clippy --all-targets  # Strict lint
cd src-tauri && cargo test     # Unit tests
```

## Architecture

```
src-tauri/                # Rust + Tauri backend
├── src/
│   ├── commands/         # Tauri IPC (agent_chat, heartbeat, api_keys, llm, forecast, mcp, gateway, ...)
│   ├── services/
│   │   ├── agent_local/  # Session store, tools, permission gate, subagents, tool results
│   │   ├── llm/          # Unified OpenAI-compatible client, catalog, SSE streaming
│   │   ├── search/       # Brave, Exa, Firecrawl + routing
│   │   ├── forecast/     # Forecast catalog, runs, scenarios, notes, exports, sidecar runtime
│   │   ├── mcp_bridge/   # MCP connector config, process manager, stdio/HTTP bridge
│   │   ├── mcp_oauth/    # OAuth callback, token storage, connector auth
│   │   ├── gateway/      # External channels, background runtime, audit log
│   │   ├── scheduler/    # Internal Tokio scheduler (wakeups)
│   │   ├── git/          # Branch ops, status, watcher, worktree listing (git2)
│   │   ├── terminal/     # Cross-platform PTY (portable-pty)
│   │   ├── file_preview/ # Text, binary, image, spreadsheet, and document previews
│   │   ├── link_preview.rs  # URL metadata preview
│   │   ├── compress.rs   # Context compression helpers
│   │   ├── codex_client.rs / codex_oauth.rs  # Codex-compatible client and OAuth
│   │   ├── paths.rs      # Centralized cross-platform data path
│   │   ├── vault.rs      # XChaCha20-Poly1305 encrypted vault
│   │   ├── api_keys.rs   # API key management (Zeroizing in memory)
│   │   ├── favorite_models.rs  # Favorite model persistence
│   │   ├── config.rs     # Tolerant config.json read/write
│   │   ├── stream_utils.rs  # Shared compute_tps, clean_think_tags
│   │   └── ollama_lifecycle.rs  # Ollama sidecar management
│   ├── tray.rs           # Tray icon (dynamic FR/EN labels)
│   ├── storage_migration.rs  # One-shot migration from legacy CL-GO
│   ├── ollama_polling.rs # Ollama status polling
│   └── models/           # ScheduledWakeup, HeartbeatConfig schemas
└── resources/            # Icons and static resources

src/                      # React frontend
├── components/
│   ├── agent-local/      # Chat, permissions, tabs, tools, file tree, previews, forecast panel
│   ├── heartbeat/        # Wakeup grid, creation popup, details
│   ├── forecast/         # Forecast workspace, charts, scenarios, notes, model manager
│   ├── ollama/           # Modelfile editor, model browser
│   ├── personality/      # Markdown viewer
│   ├── settings/         # General, Ollama, connectors, channels, API keys, forecast, LLM, advanced
│   ├── terminal/         # Integrated PTY terminal
│   ├── api-keys/         # API key configuration
│   ├── layout/           # Sidebar, toolbar, drag region
│   └── ui/               # Reusable primitives
├── hooks/                # Logic extracted by domain
├── lib/                  # platform.ts (OS detection)
├── types/                # TS types aligned with Rust
└── i18n/                 # 7 languages (FR, EN, DE, ES, IT, JA, ZH)
```

## Local storage

Data in `~/.local/share/cl-go-dash/` on all 3 OSes:

| Path | Contents |
|---|---|
| `secrets.enc` | Encrypted vault containing API keys |
| `configured-providers.json` | Configured providers registry |
| `config.json` | Heartbeat config + scheduled_wakeups + advanced settings |
| `agent-sessions/*.json` | Local Agent conversations |
| `agent-settings.json` | Default permission mode (auto/manual/chat) |
| `agent-tabs.json` | Open tabs state |
| `projects.json` | Saved projects |
| `favorite-models.json` | Favorite model list |
| `terminal-tabs.json` | Integrated terminal tabs |
| `personality-injection.json` | Personality injection settings |
| `memory/core/*.md` | Personality files |
| `skills/` | Local skills |
| `tool-results/` | Full persisted outputs for large tool results |
| `mcp-connectors.json` | Configured MCP connectors |
| `mcp-runtime/` | MCP runtime data |
| `gateway-session-map.json` | Gateway-to-agent session links |
| `forecast-analyses/` | Saved forecast analyses |
| `forecast-notes/` | Forecast notes |
| `forecast-models/` | Installed local forecast models |
| `forecast-model-configs.json` | Forecast model configuration |
| `forecast-selected-model.json` | Last selected forecast model |
| `forecast-exports/` | Forecast exports |
| `ollama-custom-models.json` | Custom Ollama model metadata |
| `logs/wakeups.jsonl` | Wakeup execution history (rolling 500 lines) |
| `logs/gateway-audit.jsonl` | Gateway audit log |
| `logs/ollama-sidecar.log` | Ollama sidecar stderr logs (overwritten on each startup) |

## Ollama — bundled sidecar

The application bundles **Ollama** as a sidecar to avoid external dependencies:

- On first launch, a setup screen downloads Ollama automatically into `~/.local/share/cl-go-dash/ollama-bundle/`
- On startup, the app checks whether an Ollama daemon is already running on `localhost:11434`
- If yes (Ollama.app already installed), it uses it as is
- If not, it launches its own downloaded binary
- On close, the sidecar is stopped cleanly (Unix SIGTERM / Windows kill + 3s grace period)
- On Linux, automatic GPU detection (AMD → ROCm archive, Nvidia → standard archive with CUDA)

**Models are shared** with Ollama.app if it is installed (`~/.ollama/models/`).

## Security

- **Encrypted vault**: API keys encrypted with XChaCha20-Poly1305, master key in the native OS keyring
- **Zeroization**: all secrets in memory use `Zeroizing<String>`, intermediate buffers are zeroized after use
- **JS never sees a key**: no Tauri command exposes `get_api_key`
- **Path traversal**: canonicalize + starts_with on every path coming from the frontend
- **Bounded collections**: ActiveStreams (32), PTY sessions (16), messages per session (2000)
- **Filtered logs**: provider HTTP bodies truncated to 200 chars, never any secret in logs

## License

[Apache License 2.0](LICENSE)
