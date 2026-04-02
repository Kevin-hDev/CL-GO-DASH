# CL-GO-DASH — Roadmap Quest 0 (Fondations)

> Dashboard desktop pour piloter CL-GO (agent IA autonome)
> Stack: Tauri v2 + React 19 + TypeScript + Rust + Vite
> Design: /Users/kevinh/Projects/CL-GO-DASH/design/

---

## Quest 0 — Fondations (Contracts + Scaffolding)

### Mission 0.1 — Scaffolding Tauri + React

**Context:** Projet vierge dans `/Users/kevinh/Projects/CL-GO-DASH`. On utilise Tauri v2
(Rust backend, WebView frontend). React 19 + Vite + TypeScript. Le dossier `design/` contient
les tokens CSS et le mockup HTML validé. Convention Kevin : 200 lignes max par fichier,
1 responsabilité par fichier, snake_case Rust, camelCase TypeScript.

**Objective:** Projet Tauri fonctionnel qui s'ouvre en fenêtre desktop vide.

**Tasks:**
1. Init Tauri v2 + React + TS avec `npm create tauri-app@latest` (ou manuellement).
   Structure : `src/` (frontend), `src-tauri/` (backend Rust). Configurer `vite.config.ts`
   avec alias `@/` → `src/`. Configurer `tsconfig.json` avec le même alias.
2. Configurer `src-tauri/capabilities/default.json` avec les permissions :
   `fs:read` sur `~/.local/share/cl-go/`, `~/.claude/projects/-Users-kevinh-Projects/`,
   `shell:allow-execute` pour `crontab`, `open`. Valider regex sur les arguments shell.
3. Configurer `tauri.conf.json` : titre "CL-GO", taille fenêtre 1200x800, titlebar native macOS.

**Files created:** `package.json`, `vite.config.ts`, `tsconfig.json`, `src/main.tsx`, `src/App.tsx`,
`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`,
`src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
**Files modified:** —
**Depends on:** —
**Validation:** `cd /Users/kevinh/Projects/CL-GO-DASH && npm run tauri dev` lance la fenêtre

---

### Mission 0.2 — Types partagés + Config reader

**Context:** Projet Tauri scaffoldé (mission 0.1). Le backend doit lire
`~/.local/share/cl-go/config.json` qui contient la config heartbeat, hooks, communication.
Les types doivent être définis côté Rust (serde) ET côté TypeScript (miroir exact).
IPC via `invoke()` (frontend) ↔ `#[tauri::command]` (backend), tout transite en JSON.

**Objective:** Types partagés + commande IPC `get_config` fonctionnelle.

**Tasks:**
1. Créer les modèles Rust : `ClgoConfig`, `HeartbeatConfig`, `CommunicationConfig`
   avec `serde::Serialize + Deserialize`. Créer `services/config.rs` qui lit/écrit
   `config.json` avec flock pour éviter la corruption concurrente.
2. Créer les types TypeScript miroir dans `src/types/`. Un fichier par domaine :
   `config.ts`, `session.ts`, `personality.ts`.
3. Créer la commande `get_config` dans `commands/config.rs`, l'enregistrer dans
   `lib.rs` via `generate_handler!`. Créer `src/services/config.ts` côté frontend
   avec `invoke('get_config')`.

**Files created:** `src-tauri/src/models/config.rs`, `src-tauri/src/models/mod.rs`,
`src-tauri/src/services/config.rs`, `src-tauri/src/services/mod.rs`,
`src-tauri/src/commands/config.rs`, `src-tauri/src/commands/mod.rs`,
`src/types/config.ts`, `src/types/session.ts`, `src/types/personality.ts`,
`src/services/config.ts`
**Files modified:** `src-tauri/src/lib.rs`
**Depends on:** 0.1
**Validation:** `cd src-tauri && cargo check` + frontend appelle `get_config` sans erreur

---

### Mission 0.3 — Design system + Layout 3 colonnes

**Context:** Design validé dans `design/` : tokens.css (palette Warm Control, Geist Sans,
grille 8px), sidebar 56px→200px au hover, liste 260px, détail flexible.
Thème dark (#131316 void, #1A1A1E shell, #222228 surface) et light (#F2F2F5, #EAEAEF, #FFFFFF).
Grain SVG noise en overlay. Transitions 200ms ease-out.

**Objective:** Layout 3 colonnes fonctionnel avec dark/light theme.

**Tasks:**
1. Porter les fichiers CSS du mockup vers `src/styles/` : `tokens.css` (variables),
   `dark.css`, `light.css`, `global.css` (reset + grain). Charger Geist Sans via
   Google Fonts ou local.
2. Créer les composants layout React : `app-layout.tsx` (conteneur flex 3 colonnes),
   `sidebar.tsx` (navigation 3 onglets + theme toggle, expand au hover),
   `list-panel.tsx` (conteneur générique), `detail-panel.tsx` (conteneur générique).
3. Brancher le routing : état `activeTab` dans App, passer aux composants layout.
   Pas de react-router — simple state switching (3 onglets seulement).

**Files created:** `src/styles/tokens.css`, `src/styles/dark.css`, `src/styles/light.css`,
`src/styles/global.css`, `src/components/layout/app-layout.tsx`,
`src/components/layout/sidebar.tsx`, `src/components/layout/list-panel.tsx`,
`src/components/layout/detail-panel.tsx`
**Files modified:** `src/App.tsx`, `src/main.tsx`
**Depends on:** 0.1
**Validation:** `npm run tauri dev` → layout 3 colonnes visible, dark/light toggle fonctionne

---

### Mission 0.4 — Hooks réutilisables (keyboard, theme, click-outside)

**Context:** Convention : Escape ferme les modales/menus, Enter valide, clic extérieur ferme.
Ces comportements sont requis PARTOUT dans l'app. Le theme toggle existe dans sidebar.tsx
(mission 0.3). Le menu contextuel (clic droit) est utilisé dans Heartbeat et Historique.

**Objective:** Hooks et composants UI réutilisables pour toute l'app.

**Tasks:**
1. `use-keyboard.ts` : hook qui écoute keydown, expose `useKeyboard({ onEscape, onEnter })`.
   `use-click-outside.ts` : hook ref-based qui appelle un callback au clic extérieur.
   `use-theme.ts` : hook qui lit/écrit le thème dans localStorage + data-theme sur html.
2. `context-menu.tsx` : composant React positionné en absolu au clic droit, utilise
   `use-click-outside` + `use-keyboard(onEscape)` pour se fermer. Props : items array
   (label, onClick, danger?). Style depuis tokens.css.

**Files created:** `src/hooks/use-keyboard.ts`, `src/hooks/use-theme.ts`,
`src/hooks/use-click-outside.ts`, `src/components/ui/context-menu.tsx`
**Files modified:** `src/components/layout/app-layout.tsx` (brancher useTheme)
**Depends on:** 0.3
**Validation:** `npx tsc --noEmit` + Escape/Enter/clic extérieur testables manuellement
