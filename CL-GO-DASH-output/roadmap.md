# CL-GO-DASH — Roadmap Phase 1 (Reprise)

> Migration du dashboard vers shadcn/ui + Tailwind, 3 thèmes, streaming temps réel, traduction anglais.
> Stack: Tauri v2 + React 19 + TypeScript + Vite + shadcn/ui + Tailwind CSS
> Brainstorm: docs/brainstorm.md
> Design ref: /Users/kevinh/Projects/Chill_Desk/_bmad-output/planning-artifacts/ux-design-specification.md

---

## Quest 0 — Fondations (Stack + Tokens + Thèmes)

### Mission 0.1 — Installer Tailwind + shadcn/ui + icônes

**Context:** CL-GO-DASH est un dashboard Tauri v2 + React 19 + TypeScript + Vite. Le frontend utilise actuellement du CSS maison (tokens.css, dark.css, light.css, global.css = 155 lignes). Pas de Tailwind, pas de design system. Les composants sont écrits à la main. La migration vers shadcn/ui nécessite Tailwind CSS comme fondation. La bibliothèque d'icônes actuelle doit être remplacée par Lucide (par défaut avec shadcn/ui).

**Objective:** Installer et configurer Tailwind CSS v4, shadcn/ui, et Lucide Icons. Le build doit compiler sans erreur. Les anciens CSS restent en place temporairement (suppression progressive dans Quest 1).

**Tasks:**
1. Installer les dépendances : `tailwindcss`, `@tailwindcss/vite`, `lucide-react`, `class-variance-authority`, `clsx`, `tailwind-merge`. Créer `tailwind.config.ts` avec les chemins src/**/*.tsx et les custom colors mappées sur les CSS Custom Properties existantes. Créer `src/lib/utils.ts` avec la fonction `cn()`.
2. Initialiser shadcn/ui : `npx shadcn@latest init`. Configurer `components.json` avec le style "new-york", les alias `@/components` et `@/lib`. Vérifier que `vite.config.ts` a le resolve alias `@` → `src/`.
3. Installer Lucide Icons (`lucide-react`). Créer un fichier `src/components/ui/icons.ts` qui re-exporte les icônes utilisées dans le projet (remplacer les icônes inline actuelles). Lister les icônes nécessaires en scannant les composants existants.

**Files created:** `tailwind.config.ts`, `postcss.config.js`, `src/lib/utils.ts`, `components.json`, `src/components/ui/icons.ts`
**Files modified:** `package.json`, `vite.config.ts`, `src/styles/global.css` (ajout directives Tailwind)
**Depends on:** —
**Validation:** `npx tsc --noEmit && cd src-tauri && cargo check`

---

### Mission 0.2 — Design tokens unifié + structure 3 thèmes

**Context:** CL-GO-DASH a actuellement 2 thèmes (dark/light) avec des tokens CSS custom dans tokens.css (spacing, radius, fonts) et des couleurs par thème dans dark.css/light.css. shadcn/ui utilise un système de tokens CSS similaire mais avec des noms différents (--background, --foreground, --primary, --muted, etc.). Il faut unifier les deux systèmes en gardant la compatibilité shadcn/ui. L'objectif final est 3 thèmes : light, dark (Chill_Desk avec orange au lieu de violet), orange.

**Objective:** Créer un système de tokens unifié compatible shadcn/ui avec 3 fichiers de thèmes. Le changement de thème se fait via la classe CSS sur `<html>`.

**Tasks:**
1. Restructurer `tokens.css` pour inclure les variables shadcn/ui requises (--background, --foreground, --card, --popover, --primary, --secondary, --muted, --accent, --destructive, --border, --input, --ring) en plus des tokens existants (spacing, radius, font). Garder les anciens noms comme alias pour la transition.
2. Créer 3 fichiers thèmes dans `src/styles/themes/` : `light.css`, `dark.css`, `orange.css`. Chaque fichier redéfinit toutes les variables couleur. Pour le moment, remplir avec des valeurs placeholder cohérentes.
3. Mettre à jour `global.css` pour importer les 3 thèmes. Mettre à jour `use-theme.ts` pour supporter 3 valeurs au lieu de 2 (light/dark/orange). L'attribut `data-theme` sur `<html>` contrôle le thème actif.

**Files created:** `src/styles/themes/light.css`, `src/styles/themes/dark.css`, `src/styles/themes/orange.css`
**Files modified:** `src/styles/tokens.css`, `src/styles/global.css`, `src/hooks/use-theme.ts`
**Depends on:** 0.1
**Validation:** `npx tsc --noEmit`

---

### Mission 0.3 — Extraire les tokens Chill_Desk (orange + dark)

**Context:** Le projet Chill_Desk dans `/Users/kevinh/Projects/Chill_Desk/` utilise shadcn/ui + Tailwind avec un système de thèmes défini dans ses CSS. Il a un thème orange et un thème dark (violet). On veut reprendre le thème orange tel quel et le thème dark en remplaçant le violet par le même orange. Les fichiers de référence sont dans le dossier `src/styles/` ou `app/globals.css` de Chill_Desk.

**Objective:** Remplir les fichiers thèmes créés en 0.2 avec les vraies valeurs extraites de Chill_Desk. Le thème orange est une copie directe. Le thème dark prend le dark de Chill_Desk mais remplace toutes les teintes violet/purple par les teintes orange correspondantes.

**Tasks:**
1. Lire les fichiers CSS de Chill_Desk pour extraire les tokens du thème orange. Copier les valeurs dans `src/styles/themes/orange.css`. Adapter les noms de variables si nécessaire pour matcher le système CL-GO-DASH.
2. Lire le thème dark de Chill_Desk. Copier dans `src/styles/themes/dark.css`. Identifier toutes les valeurs violet/purple (teinte ~270-290 en HSL) et les remplacer par les valeurs orange correspondantes (même saturation/luminosité, teinte ~25-35).
3. Vérifier visuellement les 3 thèmes en lançant l'app (`npm run tauri dev`) et en basculant entre eux. Ajuster les contrastes si nécessaire pour garantir la lisibilité.

**Files created:** —
**Files modified:** `src/styles/themes/dark.css`, `src/styles/themes/orange.css`, `src/styles/themes/light.css` (ajustements si nécessaire)
**Depends on:** 0.2
**Validation:** `npx tsc --noEmit`

---

## Quest 1 — Composants UI (Migration visuelle)

### Mission 1.1 — Migrer layout (sidebar + panels)

**Context:** CL-GO-DASH utilise un layout 3 colonnes maison : sidebar (src/components/layout/sidebar.tsx) + list panel + detail panel. Le système est piloté par un `useState<TabId>` dans App.tsx avec 3 onglets (heartbeat, history, personality). Chaque onglet expose `{ list, detail }` via une "tab function". La sidebar a un toggle dark/light en bas. Il faut migrer vers les composants shadcn/ui tout en gardant le même pattern architectural (tab functions). Les icônes inline doivent être remplacées par Lucide.

**Objective:** Remplacer les composants layout maison par des composants shadcn/ui (Sidebar, Button, Separator). Remplacer les icônes inline par Lucide. Le layout doit fonctionner identiquement mais avec le nouveau design system.

**Tasks:**
1. Installer les composants shadcn/ui nécessaires : `npx shadcn@latest add sidebar button separator tooltip`. Adapter `sidebar.tsx` pour utiliser le composant Sidebar shadcn/ui avec les 3 nav items + un slot en bas pour Settings (Quest 2) et le theme toggle. Remplacer les icônes par Lucide (Activity, Clock, User, Settings, Sun, Moon).
2. Migrer `list-panel.tsx` et `detail-panel.tsx` vers Tailwind classes (supprimer le CSS maison correspondant). Garder le pattern tab function intact.
3. Migrer `App.tsx` : supprimer les imports CSS maison devenus inutiles, utiliser les classes Tailwind pour le layout grid. Supprimer les fichiers CSS qui ne sont plus utilisés après cette migration.

**Files created:** `src/components/ui/sidebar.tsx`, `src/components/ui/button.tsx`, `src/components/ui/separator.tsx`, `src/components/ui/tooltip.tsx`
**Files modified:** `src/components/layout/sidebar.tsx`, `src/components/layout/app-layout.tsx`, `src/components/layout/list-panel.tsx`, `src/components/layout/detail-panel.tsx`, `src/App.tsx`
**Depends on:** 0.3
**Validation:** `npx tsc --noEmit`

---

### Mission 1.2 — Migrer heartbeat tab

**Context:** Le heartbeat tab (src/components/heartbeat/) est le module le plus complexe du dashboard : liste des wakeups avec sub-tabs "planned"/"warning", formulaire de création/édition (time, mode, prompt), mode selector (4 boutons auto/explorer/free/evolve), signal dot (idle/live/error/ok), warnings panel. Il utilise le hook `use-heartbeat.ts` pour le state. Les composants utilisent du CSS maison pour les styles.

**Objective:** Migrer tous les composants heartbeat vers shadcn/ui + Tailwind. Ajouter la fonctionnalité de renommage inline des wakeups.

**Tasks:**
1. Installer : `npx shadcn@latest add card badge input select switch`. Migrer `heartbeat-list.tsx` : remplacer les styles maison par Card + Badge + Switch (pour le toggle heartbeat). Migrer `mode-selector.tsx` vers des Button shadcn avec variants.
2. Migrer `heartbeat-detail.tsx` : utiliser Card pour le container, Input pour time/prompt, Select pour le mode. Migrer `signal-dot.tsx` et `warnings.tsx` vers Tailwind.
3. Ajouter le renommage inline des wakeups : champ `name` optionnel dans le modèle ScheduledWakeup (backend `models/config.rs`), input inline dans la liste (même pattern que le rename sessions dans history), commande IPC `update_wakeup` accepte déjà les updates.

**Files created:** `src/components/ui/card.tsx`, `src/components/ui/badge.tsx`, `src/components/ui/input.tsx`, `src/components/ui/select.tsx`, `src/components/ui/switch.tsx`
**Files modified:** `src/components/heartbeat/*.tsx`, `src-tauri/src/models/config.rs`, `src/types/config.ts`
**Depends on:** 1.1
**Validation:** `npx tsc --noEmit && cd src-tauri && cargo check`

---

### Mission 1.3 — Migrer history tab

**Context:** Le history tab (src/components/history/) affiche les sessions passées avec sub-tabs "recent" (30 dernières) / "archive" (31-60). Le detail montre les messages de la conversation, les outils utilisés, les fichiers modifiés, avec une animation Lottie "thinking" pour les sessions en cours. Il y a un export markdown et un rename inline. Le hook `use-sessions.ts` gère le state avec auto-refresh via FS events.

**Objective:** Migrer tous les composants history vers shadcn/ui + Tailwind. Garder Lottie pour l'animation thinking.

**Tasks:**
1. Installer : `npx shadcn@latest add scroll-area dialog tabs`. Migrer `history-list.tsx` : utiliser Card pour chaque session, Badge pour le mode, Tabs pour recent/archive. Garder le rename inline existant.
2. Migrer `session-detail.tsx` : utiliser Card pour la meta grid, ScrollArea pour la conversation, Dialog pour la confirmation de suppression. Garder Lottie tel quel. Remplacer le CSS maison par Tailwind.
3. Supprimer les fichiers CSS history-specific qui ne sont plus utilisés.

**Files created:** `src/components/ui/scroll-area.tsx`, `src/components/ui/dialog.tsx`, `src/components/ui/tabs.tsx`
**Files modified:** `src/components/history/*.tsx`
**Depends on:** 1.1
**Validation:** `npx tsc --noEmit`

---

### Mission 1.4 — Migrer personality tab + markdown viewer

**Context:** Le personality tab (src/components/personality/) affiche les fichiers core/ et inbox/ en lecture seule avec un markdown viewer maison (markdown-viewer.tsx). Le viewer utilise `dangerouslySetInnerHTML` avec une fonction `esc()` pour l'échappement — fragile. Il ne supporte pas les tables, code blocks, images, liens, listes imbriquées.

**Objective:** Migrer vers shadcn/ui + Tailwind. Remplacer le markdown viewer maison par une solution plus robuste (react-markdown ou marked avec sanitize).

**Tasks:**
1. Migrer `personality-tab.tsx` et `personality-list.tsx` vers shadcn/ui (Card, ScrollArea). Supprimer le CSS maison.
2. Remplacer `markdown-viewer.tsx` : installer `react-markdown` + `remark-gfm` pour le rendu markdown complet (tables, code, liens, listes). Styler avec les classes Tailwind `prose` (plugin `@tailwindcss/typography`). Supprimer le `dangerouslySetInnerHTML`.
3. Installer `@tailwindcss/typography` et configurer dans `tailwind.config.ts`.

**Files created:** —
**Files modified:** `src/components/personality/*.tsx`, `package.json`, `tailwind.config.ts`
**Depends on:** 1.1
**Validation:** `npx tsc --noEmit`

---

### Mission 1.5 — Migrer composants partagés (toast, context-menu, empty-state, skeleton)

**Context:** CL-GO-DASH a des composants UI partagés maison : ToastProvider + Toast (notifications avec check animation), ContextMenu (clic-droit positionné), EmptyState (placeholder), Skeleton (loading lines), DatetimeInput, ErrorBoundary. Le ToastProvider est sous-utilisé (les erreurs vont dans console.error au lieu des toasts).

**Objective:** Remplacer par les équivalents shadcn/ui. Brancher les erreurs sur le système de toast.

**Tasks:**
1. Installer : `npx shadcn@latest add toast dropdown-menu skeleton`. Migrer Toast/ToastProvider vers shadcn/ui toast (utilise Sonner). Migrer ContextMenu vers DropdownMenu.
2. Migrer EmptyState et DatetimeInput vers Tailwind. Supprimer les fichiers CSS maison correspondants.
3. Brancher les erreurs des hooks (`use-heartbeat.ts`, `use-sessions.ts`) sur le toast au lieu de `console.error`.

**Files created:** `src/components/ui/toast.tsx`, `src/components/ui/dropdown-menu.tsx`, `src/components/ui/skeleton.tsx`
**Files modified:** `src/components/ui/context-menu.tsx`, `src/components/ui/empty-state.tsx`, `src/components/ui/datetime-input.tsx`, `src/hooks/use-heartbeat.ts`, `src/hooks/use-sessions.ts`
**Depends on:** 1.2 (heartbeat utilise les composants partagés)
**Validation:** `npx tsc --noEmit`

---

## Quest 2 — Settings

### Mission 2.1 — Page Settings + taille police + sélection police

**Context:** CL-GO-DASH n'a pas de page Settings. Le toggle thème est dans la sidebar. Il faut créer un nouvel onglet Settings en bas de la sidebar (au-dessus du toggle actuel) avec : slider de taille de police (5 niveaux : 100%, 112%, 125%, 137%, 150%), sélection de police (dropdown avec preview), et plus tard langue. Les préférences persistent dans localStorage.

**Objective:** Créer l'onglet Settings avec slider de taille de police et sélection de police. Les changements s'appliquent en temps réel.

**Tasks:**
1. Créer `settings-tab.tsx` avec la tab function pattern (`{ list, detail }`). Ajouter dans le NAV_ITEMS du sidebar (icône Settings de Lucide, position en bas). Créer `use-settings.ts` qui gère fontSize (100-150%), fontFamily, et persiste dans localStorage.
2. Implémenter le slider de taille de police : composant Slider shadcn/ui (`npx shadcn@latest add slider`) avec 5 stops (100, 112, 125, 137, 150). Flèches < > aux extrémités via boutons. Le changement applique `font-size` sur `:root` en temps réel.
3. Implémenter la sélection de police : Select shadcn/ui avec 4-5 polices (Geist Sans, Inter, System UI, JetBrains Mono, IBM Plex Sans). Preview de chaque police dans le dropdown. Le changement applique `--font-sans` sur `:root`.

**Files created:** `src/components/settings/settings-tab.tsx`, `src/hooks/use-settings.ts`
**Files modified:** `src/App.tsx`, `src/components/layout/sidebar.tsx`
**Depends on:** 1.1
**Validation:** `npx tsc --noEmit`

---

### Mission 2.2 — Thème selector dans Settings

**Context:** Le toggle thème est actuellement un bouton Sun/Moon dans la sidebar qui bascule entre dark et light. Avec 3 thèmes (light, dark, orange), un simple toggle ne suffit plus. Il faut déplacer la sélection dans Settings avec un preview visuel de chaque thème.

**Objective:** Remplacer le toggle sidebar par un sélecteur 3 thèmes dans Settings. Le toggle sidebar disparaît au profit d'un lien vers Settings.

**Tasks:**
1. Ajouter dans `settings-tab.tsx` une section "Theme" avec 3 cards cliquables montrant un aperçu de chaque thème (couleur de fond + accent). Le thème actif a un indicateur visuel (bordure ou check).
2. Migrer la logique de `use-theme.ts` pour supporter 3 valeurs (light/dark/orange). Supprimer le toggle Sun/Moon de la sidebar. Garder le raccourci clavier si existant.

**Files created:** —
**Files modified:** `src/components/settings/settings-tab.tsx`, `src/hooks/use-theme.ts`, `src/components/layout/sidebar.tsx`
**Depends on:** 2.1
**Validation:** `npx tsc --noEmit`

---

## Quest 3 — Streaming temps réel

### Mission 3.1 — Backend : watcher JSONL session live

**Context:** Le dashboard détecte les sessions en cours via poll PID toutes les 5s (use-session-status.ts → invoke get_session_status). Pour voir le contenu, l'utilisateur doit ouvrir le détail manuellement. Le file_watcher.rs surveille config.json, les sessions (dossier), les personality files, et les logs. Il n'y a pas de watcher sur les fichiers JSONL individuels.

**Objective:** Ajouter un watcher sur le fichier JSONL de la session en cours. Quand une nouvelle ligne est ajoutée, émettre un event Tauri avec le contenu parsé. Le frontend reçoit les nouveaux messages en temps réel.

**Tasks:**
1. Dans `file_watcher.rs`, ajouter la surveillance du fichier `session.pid` pour détecter quelle session est en cours. Quand un PID est actif, surveiller le fichier JSONL correspondant.
2. Créer un mécanisme de "tail" incrémental : stocker la position courante du fichier, lire uniquement les nouvelles lignes à chaque notification FS. Émettre un event `fs:session-message` avec les nouvelles entrées parsées (utiliser session_detail.rs pour le parsing).
3. Ajouter un debounce spécifique pour les JSONL (100ms au lieu de 200ms) car les sessions écrivent fréquemment.

**Files created:** —
**Files modified:** `src-tauri/src/services/file_watcher.rs`, `src-tauri/src/services/session_detail.rs`, `src-tauri/src/lib.rs`
**Depends on:** —
**Validation:** `cd src-tauri && cargo check`

---

### Mission 3.2 — Frontend : streaming session live + auto-refresh personality

**Context:** Le session-detail.tsx charge le détail complet via `invoke('get_session_detail')` à la demande. Le personality-tab.tsx rafraîchit la liste des fichiers sur `fs:personality-changed` mais pas le contenu du fichier ouvert. Le hook use-sessions.ts ne gère pas les messages incrémentaux.

**Objective:** Le session detail affiche les nouveaux messages en temps réel (streaming). Le personality viewer relit le contenu quand le fichier change.

**Tasks:**
1. Dans `use-sessions.ts` ou un nouveau hook `use-live-session.ts` : écouter l'event `fs:session-message`. Quand il arrive, ajouter les nouveaux messages au state sans recharger tout le détail. Auto-scroll vers le bas.
2. Dans `personality-tab.tsx` : quand `fs:personality-changed` arrive ET qu'un fichier est sélectionné → relire son contenu via `invoke('read_personality_file')`. Pas de re-clic nécessaire.
3. Ajouter un indicateur visuel "live" dans le session detail quand le streaming est actif (point vert pulsant, ou le badge "EN COURS" existant).

**Files created:** `src/hooks/use-live-session.ts` (optionnel, peut être intégré dans use-sessions.ts)
**Files modified:** `src/components/history/session-detail.tsx`, `src/components/personality/personality-tab.tsx`, `src/hooks/use-sessions.ts`
**Depends on:** 3.1
**Validation:** `npx tsc --noEmit`

---

### Mission 3.3 — Backend+Frontend : watcher monitoring + frictions

**Context:** Les fichiers `monitoring.md` et `frictions.md` dans `~/.local/share/cl-go/` ne sont pas surveillés par le file watcher. Si on ajoute un onglet ou une vue pour ces fichiers plus tard (Phase 2 features), il faudra que le watcher soit déjà en place.

**Objective:** Ajouter monitoring.md et frictions.md au file watcher. Émettre des events quand ils changent. Préparer le terrain pour les futures features sans rien afficher pour l'instant.

**Tasks:**
1. Dans `file_watcher.rs` : ajouter la surveillance de `~/.local/share/cl-go/monitoring.md` et `~/.local/share/cl-go/frictions.md`. Émettre `fs:monitoring-changed` et `fs:frictions-changed`.
2. Créer un hook `use-fs-event.ts` générique (s'il n'existe pas déjà) qui écoute n'importe quel event FS nommé et appelle un callback. Vérifier que le hook existant couvre ce cas.
3. Documenter les nouveaux events dans CLAUDE.md.

**Files created:** —
**Files modified:** `src-tauri/src/services/file_watcher.rs`, `src-tauri/src/lib.rs`, `CLAUDE.md`
**Depends on:** 3.1
**Validation:** `cd src-tauri && cargo check`

---

## Quest 4 — Traduction + Polish

### Mission 4.1 — i18n : installer i18next + extraire strings

**Context:** L'interface est actuellement en français (labels, boutons, messages). On veut la passer en anglais avec la possibilité de supporter d'autres langues plus tard. i18next + react-i18next est le standard pour React.

**Objective:** Installer i18next, créer le fichier de traduction en.json, extraire toutes les strings hardcodées des composants.

**Tasks:**
1. Installer `i18next`, `react-i18next`. Créer `src/i18n/index.ts` (config i18next, langue par défaut "en") et `src/i18n/en.json` (toutes les strings organisées par domaine : heartbeat, history, personality, settings, common).
2. Initialiser i18n dans `main.tsx`. Remplacer les strings hardcodées par `t('key')` dans TOUS les composants (scanner chaque fichier .tsx pour les textes français).
3. Créer `src/i18n/fr.json` comme copie de en.json pour garder le français en option (pour plus tard).

**Files created:** `src/i18n/index.ts`, `src/i18n/en.json`, `src/i18n/fr.json`
**Files modified:** `package.json`, `src/main.tsx`, tous les fichiers .tsx avec des strings
**Depends on:** 1.1 (les composants doivent être migrés avant de toucher les strings)
**Validation:** `npx tsc --noEmit`

---

### Mission 4.2 — Fix run_wakeup prompt custom + renommer wakeups backend

**Context:** La commande `run_wakeup` dans heartbeat.rs a un placeholder `let _ = wakeup;` — le prompt custom configuré dans le wakeup n'est pas passé au wrapper. Le modèle ScheduledWakeup n'a pas de champ `name` pour le renommage (ajouté côté Rust dans 1.2 pour le type, mais la commande IPC doit le supporter).

**Objective:** Le prompt custom est passé au wrapper lors du lancement. Le champ name est persisté et visible.

**Tasks:**
1. Dans `heartbeat.rs` `run_wakeup` : extraire le prompt du wakeup et le passer comme argument au wrapper (variable d'environnement `CLGO_WAKEUP_PROMPT` ou argument supplémentaire). Mettre à jour le wrapper pour lire et utiliser ce prompt.
2. Vérifier que `update_wakeup` et `create_wakeup` supportent le champ `name` ajouté au modèle. Tester la persistence.

**Files created:** —
**Files modified:** `src-tauri/src/commands/heartbeat.rs`, `~/.local/share/cl-go/scripts/heartbeat/go-heartbeat-wrapper.sh`
**Depends on:** 1.2
**Validation:** `cd src-tauri && cargo check`

---

### Mission 4.3 — Bug fixes : CSP, cleanup log, erreurs toast, markdown sécurisé

**Context:** Plusieurs bugs et incohérences identifiés : CSP null dans tauri.conf.json (sécurité), cleanup sessions >60 silencieux (pas de feedback), erreurs hooks en console.error (pas de toast visible). Le markdown viewer aura été remplacé en 1.4 mais la CSP et les autres points restent.

**Objective:** Durcir la sécurité et améliorer le feedback utilisateur.

**Tasks:**
1. Configurer une CSP restrictive dans `tauri.conf.json` : `default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'`. Tester que l'app fonctionne toujours.
2. Ajouter un toast (ou log visible) quand le cleanup sessions >60 s'exécute au boot : "X old sessions archived". Le toast utilise le système shadcn/ui installé en 1.5.
3. Vérifier que toutes les erreurs dans les hooks (catch blocks) sont routées vers le toast au lieu de console.error seulement. Audit rapide des fichiers hooks/*.ts.

**Files created:** —
**Files modified:** `src-tauri/tauri.conf.json`, `src/hooks/use-sessions.ts`, `src/hooks/use-heartbeat.ts`
**Depends on:** 1.5 (toast system en place)
**Validation:** `npx tsc --noEmit && cd src-tauri && cargo check`

---

## Execution order

```
Quest 0 (fondations) — séquentiel
  0.1 Stack install ─────────────── no dependency
  └─ 0.2 Tokens + thèmes ────────── depends on 0.1
     └─ 0.3 Extract Chill_Desk ──── depends on 0.2

Quest 1 (composants) — partiellement parallélisable après 0.3
  1.1 Layout ────────────────────── depends on 0.3
  ├─ 1.2 Heartbeat ──────────────── depends on 1.1
  ├─ 1.3 History ────────────────── depends on 1.1 (parallèle avec 1.2)
  ├─ 1.4 Personality ────────────── depends on 1.1 (parallèle avec 1.2/1.3)
  └─ 1.5 Composants partagés ───── depends on 1.2

Quest 2 (settings) — après layout
  2.1 Settings page ─────────────── depends on 1.1
  └─ 2.2 Theme selector ────────── depends on 2.1

Quest 3 (streaming) — parallèle avec Quest 1/2 après Quest 0
  3.1 Backend JSONL watcher ─────── no dependency (Rust only)
  └─ 3.2 Frontend streaming ─────── depends on 3.1
  └─ 3.3 Monitoring watcher ─────── depends on 3.1

Quest 4 (polish) — après Quest 1
  4.1 i18n ──────────────────────── depends on 1.1
  └─ 4.2 Prompt fix + rename ───── depends on 1.2
  └─ 4.3 Bug fixes ─────────────── depends on 1.5
```

---

## Security checklist

- [ ] CSP configurée dans tauri.conf.json (mission 4.3)
- [ ] dangerouslySetInnerHTML supprimé (mission 1.4 — react-markdown)
- [ ] Path validation maintenue pour accès fichiers (existant, vérifier)
- [ ] Atomic config writes maintenu (existant, ne pas casser)
- [ ] Pas de secrets dans le code (existant, vérifier)
- [ ] Entrées utilisateur validées avant traitement (wakeup time, prompt)
