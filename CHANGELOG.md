# Changelog

## v0.8.4 (en cours)

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

- **Codex OAuth stream persistence** — removed premature `Done` event emitted inside `consume_sse`, which violated the `stream_chat_no_done` contract. This caused the frontend to finalize the assistant message before tool results arrived, losing all tool data on restart and showing frozen spinners. Only affected the Codex OAuth path (GPT via subscription).
- **Error event now persists tool data** — stream errors during multi-turn tool calls no longer discard accumulated segments. The `error` handler now calls `finalizeStream` to build and persist the assistant message with whatever segments completed before the error.
- **Snapshot vs DB race condition** — when loading a session, a stale in-memory stream snapshot could override complete data from the database. The snapshot is now only preferred when it contains at least as many messages as the DB.
- **Tool arguments preserved through Rust round-trip** — added `args: Option<serde_json::Value>` to the Rust `ToolActivityRecord` struct. Previously, the frontend sent `args` but Rust silently dropped the unknown field, causing degraded LLM context on session reload.
- **SavedToolBubble done indicator** — fixed the completion check for saved tools: now uses `result != null || is_error != null` instead of just `is_error != null`, which caused successful tools to show a frozen spinner instead of a checkmark after reload.
- **Persist failure logging** — `persistAssistant` now logs errors and notifies subscribers on save failure instead of silently swallowing the error.
- **Multi-turn LLM context reconstruction** — new `expandSegmentsToChat` preserves the per-turn structure (one assistant+tools block per segment) when rebuilding chat history from saved segments, instead of flattening all tools into a single turn.
- **Retry exponential back-off** — increased from 2 retries (2s/4s) to 5 retries with exponential back-off (2s/4s/8s/16s/32s ≈ 62s total). Added SSE transport errors, connection closed, and response decoding errors to the retryable error list.
- **Tool orchestration order** — parallel tool executor now uses indexed slots to preserve result order. Fixes mismatched `tool_call_id` mapping for OpenAI-compatible providers.
- **`web_fetch` permission gate** — `web_fetch` removed from read-only classification and eager dispatch now checks pre-hooks before execution.
- **`glob` returns absolute paths** — consistent with `grep` behavior.
- **`read_spreadsheet` formula display** — formulas are now returned as text (e.g. `=SUM(A1:A5)`) instead of uncalculated `0.0`, using calamine's `worksheet_formula()` with correct absolute coordinates.
- **`write_spreadsheet` multi-sheet targeting** — operations now correctly target their specified sheet instead of always writing to the first sheet. Default sheet name (`Sheet1`) documented in tool schema.
- **`write_document` schema clarity** — each property now indicates which block type it applies to (e.g. "For heading only", "For table only"). Empty tables are silently skipped instead of producing empty `<w:tbl>` elements.
- **`process_image` operations optional** — `operations` is no longer required, allowing simple format conversion by just changing the file extension (e.g. PNG → WebP).

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
