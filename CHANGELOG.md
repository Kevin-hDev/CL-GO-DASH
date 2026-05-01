# Changelog

## v0.8.1

### Features

- **i18n — 5 nouvelles langues** — Allemand, Espagnol, Italien, Chinois simplifié et Japonais (en plus de Français/Anglais)
- **Audit texte hardcodé** — tous les textes en dur dans l'UI remplacés par des clés i18n (12 fichiers corrigés, 21 nouvelles clés)
- **Dates localisées** — les mois et jours dans les réveils utilisent `Intl.DateTimeFormat` (support automatique de toutes les langues)
- **Langue de réponse du LLM** — nouveau setting dans General pour choisir dans quelle langue le modèle doit répondre (injecté dans le system prompt)
- **Settings réorganisés** — "Lancer au démarrage" et "Démarrage masqué" déplacés de Advanced vers General
- **`patch_advanced_settings`** — nouvelle commande Tauri pour la mise à jour partielle de la config

## v0.8.0

### Features

- **Context compression** — automatic and manual (`/compress`) conversation compression when token threshold is reached
- **Compression settings** — enable/disable toggle and threshold slider (0-100%, default 85%) in Settings > Advanced
- **Model eligibility** — compression available for models with native context >= 128k tokens
- **Dynamic architecture detection** — reads context length from any Ollama model architecture (Gemma, Qwen, LLaMA, Mistral, etc.)
- **All providers supported** — works with Ollama, Anthropic, OpenAI, Groq, Gemini and all OpenAI-compatible APIs
- **Post-response compression** — threshold check after each LLM response, not just before
- **Last response preserved** — the most recent LLM response is always kept visible after compression
- **Compression animation** — orange pulsing "Compression" indicator with Lottie loader at bottom of chat

### Fixes

- **Token counting** — context ring now uses real Ollama token count (`context_tokens` = last prompt + eval) instead of accumulating prompt tokens across requests
- **Per-message token display** — shows output tokens for that response only, not total context
- **Context window detection** — correctly reads `OLLAMA_CONTEXT_LENGTH` env var when no modelfile `num_ctx` is set

## v0.7.9

- **File Preview Panel** — side panel to view files created/edited by the agent (syntax highlighting, diffs, fullscreen, resize, open in external editor)
- **Syntax highlighting** in chat tool bubbles (37 languages)
- **Real line numbers** in edit diffs (shows actual file position)
- **Auto word-wrap** — text files wrap, code files scroll horizontally
- **File extension icons** (20+ types)
- Consistent diff colors and tool bubble width across chat and panel

## v0.7.6

### Features

- **Per-session permission mode**: each conversation now has its own mode (Chat/Manual/Auto) independent of others
- **Ollama model updates preserve customizations**: system prompt and parameters are saved before pull and restored after
- **Splash screen**: app icon displayed on themed background while the app loads
- **Single instance**: prevents opening duplicate windows when double-clicking the app icon (macOS/Linux/Windows)

### UI / Theming

- **Dark theme**: translucent background applied to model selector dropdown, permission mode dropdown, project directory dropdown, heartbeat cards/dialog/button, settings cards/selects, API connectors modal/cards, and Ollama modelfile raw block
- **Dark theme**: model selector provider and favorites headers now transparent (no opaque shell background)
- **Dark theme**: removed border on model selector search input and API connectors search input
- **Light theme**: user message bubbles use translucent gray (0.45 opacity)
- **Light theme**: chat input uses translucent gray background (0.80 opacity)
- **Settings subtabs**: added hover effect on mouse over
- **Sidebar**: settings icon and text now match the color of other nav items
- **Model selector dropdown**: opens to the right instead of left to avoid sidebar overlap
- **Permission mode dropdown**: removed "Mode" header line, Chat mode color changed to thinking blue (#4A9EE8)
- **API connectors modal**: fixed size (85vh) with top-aligned grid to prevent layout shift between tabs
- **Ollama Modelfile tab**: extended active tab indicator by 3px for visual balance
- **Ollama parameters editor**: `num_ctx` and `num_predict` rows shown by default

## v0.7.5

### UI / Theming

- **Dark theme**: lightened sidebar background (`--shell`) for better contrast
- **Dark theme**: chat input and user message bubbles now use translucent backgrounds (0.55 opacity)
- **Dark theme**: all sidebar hover and selection states switched to translucent white for a softer, more cohesive look
- **Light theme**: sidebar background shifted from warm beige to neutral light gray
- **Light theme**: chat background (`--void`) lightened to a clean off-white without being too bright
- **Light theme**: user message bubbles now use a translucent gray (0.45 opacity)
- **Light theme**: chat input uses a translucent gray background (0.80 opacity)
- **Light theme**: accent orange lightened across all buttons for a fresher appearance
- **Ollama tab**: extended Modelfile active tab indicator by 3px for visual balance

## v0.7.4

### Security

- Environment variable logging restricted to an explicit allowlist
- Level 3 security audit — 15 fixes covering secrets handling, input validation, error messages, and bounded collections

## v0.7.3

### Features

- Ollama sidecar: dynamic port allocation, environment variable passthrough, GPU status detection, retry logic

### Fixes

- 4 issues from GPT review + refactored files exceeding 200 lines

## v0.7.2

### Features

- Reliable hover actions, aligned icons, Ollama pull cancellation with cleanup
- Partial content and tool results preserved on stream stop

### Fixes

- Race condition: cancel now targets the correct stream token after stop+restart
- PID file to kill orphan Ollama sidecars on restart
- Toolbar alignment, model selector live refresh, CSP images

## v0.7.1

### Fixes

- Windows: 3px window border padding

## v0.7.0

### Fixes

- Windows: personality toggles fix

### Features

- Settings: CPU/GPU hardware acceleration toggle + Ollama restart

## v0.6.9

### Fixes

- Windows: update detection + NSIS installer

## v0.6.8

### Features

- Vulkan auto-enabled for AMD GPUs on Windows + sidecar logs

## v0.6.7

### Fixes

- Robust Ollama download + extraction validation (Windows fix)
