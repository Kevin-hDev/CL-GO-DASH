# Changelog

## v0.8.7

### Features

- **File tree panel** — browsable project directory tree alongside the file preview panel
- **Git branch selector** — dropdown in chat toolbar to view, search, and switch branches with real-time updates via file watcher
- **Branch conflict dialog** — when switching with uncommitted files, shows dirty file list with real diff stats (+/-) and a "commit & switch" option that auto-commits a WIP save
- **Inline branch creation** — create and checkout a new branch directly from the selector dropdown
- **Worktree navigation** — click a worktree in the branch list to switch the active project to that directory
- **Git context for the agent** — branch name and dirty count injected into the LLM system prompt, plus `create_branch` and `checkout_branch` tools (gated, require user approval)
- **Branch bubble** — centered chat bubble when the agent creates or switches branches

---

## v0.8.6

### Features

- **Subagent system** — the main agent can spawn autonomous explorer (read-only) and coder (isolated git worktree) subagents that run in the background. Results are auto-synthesized when all subagents complete.
- **Subagent accordion** — live panel above chat input showing active subagents with per-agent timers and stop buttons
- **Subagent bubble** — collapsible completion bubble in chat history with links to open subagent sessions in new tabs

### Improvements

- Structured English system prompts for subagents with XML tags and web research guidelines
- `delegate_task` tool with prompt structuring guidance and anti-duplication instructions
- Bounded spawn channel, prompt size limits, session ID validation, path traversal protection
- Worktree auto-cleanup after subagent execution via `git worktree remove`
- Guard cleanup pattern: registry + session + worktree guaranteed even on error
- `run_id` tracking across spawn/completion events for reliable multi-run isolation
- Web search/fetch tool bubbles collapsed by default
- i18n for all subagent UI in 7 languages

---

## v0.8.5

### Features

- **Working indicator** — persistent Lottie loader + "Working for Xs" with live token count shown during all streaming gaps (between segments, after tool results, waiting for first token). Timer never resets between gaps.
- **Thinking shimmer** — "Thinking" label shimmers while the model is actively thinking, stops when done
- **Tool spinner fix** — `@keyframes spin` was missing from CSS, tool bubble spinners now rotate correctly

### Improvements

- Lottie loader recolored to theme orange via CSS filter (dark/light aware)
- Streaming timer unified — both working indicator and thinking stats share the same continuous timer from stream start

---

## v0.8.4

### Features

- **Système de connecteurs MCP** — 18 connecteurs pour services externes (Notion, Slack, Linear, Reddit, HuggingFace, etc.) accessibles au LLM via un meta-tool unique `search_mcp_tools` (~80 tokens en contexte)
- **OAuth 2.1 complet** — PKCE S256, Dynamic Client Registration, discovery automatique, callback server local, refresh automatique avec mutex anti-race
- **Transport stdio pour MCP locaux** — Context7, HuggingFace, iMessage, ProductHunt, Reddit via process spawn (npx/uvx/deno) + stdin/stdout NDJSON
- **Trait McpTransport unifié** — interface commune HTTP/stdio, extensible pour futurs transports
- **ProcessManager** — pool borné max 8 process, TTL 10 min, lazy spawn, crash recovery, stderr drain
- **UI Settings → Connectors** — browse catalog 18 MCP, config tokens, OAuth auto, toggles chat
- **Menu chat connecteurs** — dropdown "+" avec sous-menu toggles par connecteur
- **Link previews in chat** — URLs in messages display rich preview cards (title, description, OG image, favicon, site name). Powered by a Rust backend that fetches and parses Open Graph metadata. YouTube videos get dedicated previews via the public oEmbed API (thumbnail + channel name). Previews are deduplicated, capped at 5 per message, and grouped at the bottom of the message bubble. Toggleable in Settings > General (7 languages supported).

- **Keyboard arrow navigation** — navigate between sidebar tabs (ArrowUp/Down) and list panel items (sub-tabs, sessions, wakeups, personality files) using arrow keys. ArrowLeft/Right switches focus between sidebar and list panel. Does not interfere with existing shortcuts (Cmd+arrows for history) or text inputs.

### Fixes

- **Codex OAuth persistence** — fixed premature `Done` event that caused tool data loss and frozen spinners on GPT sessions
- **Stream error recovery** — errors during multi-turn tool calls now persist completed segments instead of discarding them
- **Session reload race** — stale stream snapshot no longer overrides complete DB data on session load
- **Tool arguments round-trip** — `args` field now preserved through Rust serialization (was silently dropped)
- **Tool completion indicator** — saved tools show ✓/✗ correctly instead of frozen spinner after reload
- **Persist failure logging** — save errors are now logged and reported instead of silently swallowed
- **Multi-turn context** — chat history reconstruction preserves per-turn structure instead of flattening all tools
- **Retry back-off** — 5 retries with exponential back-off (2s→32s, ~62s total), SSE transport errors now retryable
- **Parallel tool order** — indexed slots preserve result order, fixes `tool_call_id` mapping for OpenAI-compat
- **`web_fetch` permission gate** — no longer classified as read-only, eager dispatch checks pre-hooks
- **`glob`** — returns absolute paths (consistent with `grep`)
- **`read_spreadsheet`** — formulas returned as text instead of `0.0`
- **`write_spreadsheet`** — operations target correct sheet, default `Sheet1` documented
- **`write_document`** — schema clarified per block type, empty tables skipped
- **`process_image`** — `operations` now optional for simple format conversion

### Security

- **Permission gate MCP** — `search_mcp_tools` mode "call" nécessite approbation utilisateur
- **Sérialisation request/response stdio** — `request_lock` empêche le mélange de réponses entre appels concurrents
- **Endpoint HTTP validé** — liste de domaines de confiance, pas d'URL arbitraire
- **Spawn sécurisé** — whitelist programmes (npx/uvx/deno), regex args, env_clear + env minimal, blocklist env_keys
- **Sanitisation tools MCP** — noms 64 chars, descriptions 250 chars, schemas profondeur 4 / 20 props
- **bounded_json OAuth** — réponses OAuth/discovery limitées à 512 KB
- **Mutex refresh token** — pas de race condition sur le refresh simultané
- **Tokens résolus au spawn** — pas stockés en mémoire dans la struct transport
- **Cache invalidé** — à la suppression de token OAuth ou env
- **Erreurs MCP sanitisées** — 200 chars max, control chars filtrés
- **notifications/initialized fail closed** — erreur bloque au lieu de laisser passer

## v0.8.3

### Features

- **3 new LLM providers** — xAI (Grok 4.x), Moonshot (Kimi K2.6) and Z.ai (GLM-5.1) added to the unified OpenAI-compatible backend. Static model catalogs with context length for providers without `/models` endpoint.
- **Grok 4.3** — latest xAI model added (1M context, native reasoning, vision)
- **Updated provider descriptions** — OpenAI updated to GPT-5.5, DeepSeek updated to V4-Flash/V4-Pro
- **Multi-turn reasoning** — thinking/reasoning content now persists across tool calls in chat sessions
- **Moonshot balance API** — quota display for Moonshot Kimi via `/v1/users/me/balance`
- **Provider capability detection** — per-provider modules for tools, thinking and vision detection (xAI, Moonshot, Z.ai)

### Security

- **Test-before-save for API keys** — keys are now tested before being saved to the vault. Invalid keys are never persisted. New `test_api_key_with_value` command tests without storing.
- **Vault base64 zeroization** — master key base64 strings from keyring read/write are now properly zeroized after use
- **IPC key zeroization** — API key strings from Tauri IPC are zeroized after being copied to the vault
- **Input validation** — provider ID and key format validation before any vault operation, unknown providers rejected
- **Bounded parsing** — model list parsing capped at 500 entries, model name length validated (max 128 bytes)
- **Generic error messages** — no filesystem paths or stack traces exposed to the frontend
- **Log redaction** — sensitive JSON fields redacted from HTTP body logs
- **Removed unused search providers** — SerpAPI and Google CSE removed from catalog (were listed but never implemented)

## v0.8.2

### Features

- **6 tools office natives** — Le LLM peut manipuler des fichiers Excel, Word, PDF et images sans dépendance externe (calamine, rust_xlsxwriter, umya-spreadsheet, pdf-extract, image). Cross-platform macOS/Linux/Windows.
  - `read_spreadsheet` / `write_spreadsheet` — xlsx, xls, ods, xlsm, csv, tsv
  - `read_document` / `write_document` — pdf, docx
  - `read_image` / `process_image` — jpeg, png, webp (resize, crop, conversion)
- **Previews office dans les bulles du chat** — chaque appel write_spreadsheet affiche un tableau avec les numéros de lignes et lettres de colonnes Excel correspondant aux cellules écrites. Les write_document affichent les blocs de contenu (titres, paragraphes, listes, tableaux).
- **Previews office dans le panel** — rendu fidèle des fichiers dans le panel latéral :
  - Spreadsheet : table custom avec en-têtes de colonnes, numéros de lignes, sélecteur de feuilles, scroll
  - DOCX : rendu Word via docx-preview (styles, polices, tableaux)
  - PDF : rendu PDFium via EmbedPDF (fidélité Chrome)
- **Historique des modifications** — chaque écriture office sauvegarde ses opérations pour afficher le contenu tel qu'il était au moment de l'écriture, pas l'état actuel du fichier
- **Icônes fichiers office** — xlsx, xls, xlsm, csv, ods, tsv, docx, pdf dans le panel
- **Détection d'éditeurs externe** — fonctionne nativement pour tous les formats office (macOS Launch Services, Linux xdg-mime, Windows assoc)
- **Tolérance JSON des LLMs** — coercion tolérante, réparation JSON malformé, normalisation des formules françaises (SOMME→SUM, etc.), détection auto du type de valeur (nombres en string, formules, booléens)

### Security

- Collections bornées : MAX_OPS, MAX_CELLS, MAX_ROW, MAX_COL (frontend), HARD_MAX_COLS (Rust)
- Limites de taille fichier : 50 MB pour les previews binaires et spreadsheet
- Validation is_file() + whitelist d'extensions pour read_binary_preview
- Path traversal bloqué par les pré-hooks sur les 3 tools write

### Fixes

- Fix toolsToRecords pour write_spreadsheet, write_document, process_image (summary était JSON.stringify au lieu du path)
- Fix historique panel : les previews write_file montrent le snapshot sauvegardé au lieu de relire le fichier sur disque
- Suppression des previews read_ dans les bulles et le panel (pas utiles pour la lecture seule)

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
