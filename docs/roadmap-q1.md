# CL-GO-DASH — Roadmap Quest 1 (Heartbeat/Cron)

## Quest 1 — Heartbeat/Cron Management

### Mission 1.1 — Backend CRUD crons

**Context:** Tauri v2 app avec config reader (mission 0.2). Le heartbeat est géré par
`config.json` (champ `heartbeat` : active, mode, stop_at, interval_minutes) et par crontab
macOS. Le wrapper script est à `~/.local/share/cl-go/scripts/heartbeat/go-heartbeat-wrapper.sh`.
Modes valides : auto, explorer, free, evolve. Le dashboard doit pouvoir planifier PLUSIEURS
réveils (pas juste un seul) — on étend `config.json` avec un champ `scheduled_wakeups: []`.

**Objective:** Commandes IPC pour lister, créer, modifier, supprimer des réveils.

**Tasks:**
1. Étendre le modèle Rust `ClgoConfig` avec `scheduled_wakeups: Vec<ScheduledWakeup>`.
   Chaque wakeup : id (uuid), time (HH:MM), mode, prompt (optionnel), active (bool),
   stop_at (optionnel). Écriture atomique config.json (write tmp + rename).
2. Créer `commands/heartbeat.rs` : `list_wakeups`, `create_wakeup`, `update_wakeup`,
   `delete_wakeup`. Chaque commande lit/écrit config.json via le service config.
3. Créer `services/cron.rs` : fonctions pour ajouter/supprimer des entrées crontab.
   Utiliser `Command::new("crontab")` avec validation regex sur les arguments.
   Chaque wakeup actif = une ligne crontab pointant vers le wrapper.

**Files created:** `src-tauri/src/commands/heartbeat.rs`, `src-tauri/src/services/cron.rs`
**Files modified:** `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs`,
`src-tauri/src/services/mod.rs`, `src-tauri/src/models/config.rs`
**Depends on:** 0.2
**Validation:** `cargo check` + invoke `list_wakeups` retourne un array JSON

---

### Mission 1.2 — Frontend liste + détail heartbeat

**Context:** Layout 3 colonnes (mission 0.3), hooks keyboard/click-outside (mission 0.4),
backend heartbeat CRUD (mission 1.1). Design mockup : liste verticale de wakeups avec
voyant signal (idle/live/error), badge mode (auto/explorer/free/evolve). Détail : formulaire
avec heure (input time), stop_at (datetime-local), mode selector (pills), prompt (textarea).
Boutons : Run, Modifier, Supprimer. Style : tokens.css + components-core.css du mockup.

**Objective:** Onglet Heartbeat complet avec liste et formulaire d'édition.

**Tasks:**
1. `heartbeat-list.tsx` : composant qui affiche les wakeups en liste verticale dans
   le list-panel. Chaque item : signal dot + titre + meta + badge mode. Item sélectionné
   avec barre accent. Bouton "+ Planifier" en bas. Clic droit → context-menu (renommer/supprimer).
2. `heartbeat-detail.tsx` : formulaire dans le detail-panel. Inputs : heure, stop_at, mode
   (mode-selector.tsx avec pills cliquables), prompt (textarea). Toggle actif/inactif.
   Status row avec voyant + durée. Boutons Run/Modifier/Supprimer compacts.
3. `src/services/heartbeat.ts` : fonctions frontend `invoke` pour chaque commande CRUD.
   State management avec `useReducer` pour la liste des wakeups.

**Files created:** `src/components/heartbeat/heartbeat-list.tsx`,
`src/components/heartbeat/heartbeat-detail.tsx`,
`src/components/heartbeat/mode-selector.tsx`, `src/services/heartbeat.ts`
**Files modified:** —
**Depends on:** 0.4, 1.1
**Validation:** `npx tsc --noEmit` + liste affichée, formulaire fonctionnel

---

### Mission 1.3 — Run manuel + Voyants d'état (PID watcher)

**Context:** Le wrapper heartbeat (`go-heartbeat-wrapper.sh`) lance `claude -p "/go --{mode}"`.
Pour le run manuel, le dashboard doit ouvrir un Terminal.app avec la commande. Pour détecter
si une session est active : créer un fichier `session.pid` au lancement, le supprimer à la fin.
Le dashboard surveille ce fichier pour mettre à jour le voyant (gris→jaune→gris/rouge).

**Objective:** Bouton Run fonctionnel + voyant temps réel.

**Tasks:**
1. Backend : commande `run_wakeup(id)` qui ouvre Terminal.app via
   `open -a Terminal.app /path/to/wrapper.sh`. Commande `get_session_status` qui
   vérifie l'existence de `session.pid` et si le PID est vivant (kill -0).
2. Modifier le wrapper (`go-heartbeat-wrapper.sh`) pour écrire `session.pid` au
   lancement et le supprimer à la fin (trap EXIT).
3. Frontend : `signal-dot.tsx` (composant voyant réutilisable avec animation pulse CSS),
   `use-session-status.ts` (hook qui poll `get_session_status` toutes les 5s quand actif).

**Files created:** `src-tauri/src/services/watcher.rs`, `src/components/heartbeat/signal-dot.tsx`,
`src/hooks/use-session-status.ts`
**Files modified:** `src-tauri/src/commands/heartbeat.rs`, `src-tauri/src/services/mod.rs`,
`~/.local/share/cl-go/scripts/heartbeat/go-heartbeat-wrapper.sh`
**Depends on:** 1.2
**Validation:** Bouton Run ouvre Terminal + voyant jaune visible pendant session

---

### Mission 1.4 — Sous-onglet Warning (logs d'erreurs)

**Context:** Les logs du wrapper sont dans `~/.local/share/cl-go/logs/heartbeat/wrapper.log`.
Format : `[YYYY-MM-DD HH:MM:SS] message`. Les erreurs commencent par "ERROR:" ou
correspondent à des sessions qui n'ont pas de "Session ended" après un "Launching".

**Objective:** Afficher les erreurs dans un sous-onglet Warning.

**Tasks:**
1. Backend : `services/log-reader.rs` qui parse `wrapper.log`, extrait les erreurs
   (lignes ERROR + sessions sans "ended"), retourne un Vec<LogEntry> avec timestamp + message.
   Commande `get_warnings` dans `heartbeat.rs`.
2. Frontend : tabs "Planifiés" / "Warning" dans le list-header du heartbeat.
   `warnings.tsx` : liste des erreurs avec border-left rouge, timestamp en mono, message.

**Files created:** `src-tauri/src/services/log-reader.rs`, `src/components/heartbeat/warnings.tsx`
**Files modified:** `src-tauri/src/commands/heartbeat.rs`, `src-tauri/src/services/mod.rs`
**Depends on:** 1.2
**Validation:** `cargo check` + sous-onglet Warning affiche les erreurs depuis wrapper.log
