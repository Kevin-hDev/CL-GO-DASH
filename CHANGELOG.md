# Changelog

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
