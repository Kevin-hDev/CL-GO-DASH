# CL-GO-DASH — Roadmap Quest 2 (Historique)

## Quest 2 — Historique des Sessions

### Mission 2.1 — Backend parser JSONL

**Context:** Les sessions Claude Code sont stockées dans
`~/.claude/projects/-Users-kevinh-Projects/*.jsonl`. Chaque fichier = 1 session.
Format : une ligne JSON par événement. Types utiles : `user` (message Kevin), `assistant`
(réponse Jackson), `system/turn_duration` (durationMs, messageCount),
`system/stop_hook_summary` (hooks exécutés), `file-history-snapshot` (fichiers modifiés).
Le premier message contient sessionId, version, timestamp de début. Fichiers de 1Ko à 8Mo.

**Objective:** Commande IPC qui retourne les métadonnées des 60 dernières sessions.

**Tasks:**
1. `services/session-parser.rs` : parser incrémental. Pour la liste, ne lire QUE les
   premières et dernières lignes de chaque fichier (métadonnées : date début/fin, sessionId,
   mode détecté depuis le premier message user, version). Pour le détail : parser complet
   lazy-loaded. Trier par date décroissante, retourner les 60 plus récentes.
2. `commands/sessions.rs` : commandes `list_sessions(limit, offset)` pour la liste,
   `get_session_detail(session_id)` pour le contenu complet (messages user/assistant,
   outils utilisés, fichiers modifiés, durée). Extraction du mode depuis le contenu
   du premier message user (chercher `/go --auto`, `--explorer`, `--free`).
3. Types Rust : `SessionMeta` (id, start, end, duration, mode, message_count, version,
   file_path), `SessionDetail` (meta + messages Vec<SessionMessage>).
   `SessionMessage` : role (user/assistant), content (String), timestamp.

**Files created:** `src-tauri/src/commands/sessions.rs`, `src-tauri/src/services/session-parser.rs`
**Files modified:** `src-tauri/src/commands/mod.rs`, `src-tauri/src/services/mod.rs`,
`src-tauri/src/lib.rs`
**Depends on:** 0.2
**Validation:** `cargo check` + invoke `list_sessions` retourne un JSON avec les sessions

---

### Mission 2.2 — Frontend liste récent + archive

**Context:** Design mockup : sidebar verticale avec sessions triées par date (récente → ancienne).
Tabs "Récent" (30 dernières) et "Archive" (31-60). Chaque item : voyant ok/error + titre +
date + durée + badge mode. Clic sur un item = détail à droite. Clic droit = renommer/supprimer.
Les sessions n'ont pas de "nom" par défaut — on extrait le premier message user comme titre,
tronqué à 30 caractères.

**Objective:** Liste des sessions avec tabs récent/archive.

**Tasks:**
1. `history-list.tsx` : composant liste avec tabs "Récent" / "Archive". Appelle
   `list_sessions(30, 0)` pour récent et `list_sessions(30, 30)` pour archive.
   Chaque item : signal-dot (ok si session terminée, error si crash détecté),
   titre (premier message tronqué ou nom custom), date formatée, durée, badge mode.
2. `session-detail.tsx` : conteneur détail avec header (titre + bouton Exporter) +
   tabs (Conversation / Résumé / Fichiers). Appelle `get_session_detail(id)` au clic.
3. `src/services/sessions.ts` : fonctions invoke + useReducer pour la liste + détail.

**Files created:** `src/components/history/history-list.tsx`,
`src/components/history/session-detail.tsx`, `src/services/sessions.ts`
**Files modified:** —
**Depends on:** 0.4, 2.1
**Validation:** `npx tsc --noEmit` + liste affichée avec sessions réelles

---

### Mission 2.3 — Vue conversation markdown

**Context:** Le détail d'une session affiche les messages dans un style conversationnel (pas
du JSON brut). Messages user : bulle avec fond pulse-muted, aligné à droite. Messages
assistant : bulle avec fond surface, aligné à gauche. Le code inline doit être en Geist Mono.
Au-dessus de la conversation : 4 meta-cards (durée, messages, mode, version).

**Objective:** Rendu propre des conversations et méta-données de session.

**Tasks:**
1. `conversation-view.tsx` : composant qui prend un array de SessionMessage et les rend
   en bulles alternées. Supporter le markdown basique dans le contenu (gras, code, liens).
   Utiliser une lib légère type `react-markdown` ou parser simple maison.
2. `session-meta.tsx` : grille de 4 meta-cards (durée, nombre messages, mode, version Claude).
   Style : grid auto-fit minmax(120px, 1fr), cards avec surface + edge + shadow.

**Files created:** `src/components/history/conversation-view.tsx`,
`src/components/history/session-meta.tsx`
**Files modified:** —
**Depends on:** 2.2
**Validation:** `npx tsc --noEmit` + conversation lisible avec bulles user/assistant

---

### Mission 2.4 — Renommer/supprimer + auto-cleanup >60

**Context:** Chaque session peut être renommée (nom custom stocké dans un fichier
`session-names.json` local au dashboard) ou supprimée (suppression du .jsonl).
Clic droit → context-menu. Couleur rouge foncé pour supprimer. Confirmation avant suppression.
Les sessions au-delà de 60 sont supprimées automatiquement au lancement de l'app.

**Objective:** Gestion complète des sessions (renommer, supprimer, auto-cleanup).

**Tasks:**
1. Backend : `rename_session(id, name)` écrit dans `session-names.json` (mapping id→name).
   `delete_session(id)` supprime le fichier .jsonl. `cleanup_old_sessions()` supprime
   les sessions au-delà de 60 (appelée au démarrage de l'app).
2. Frontend : intégrer le context-menu (composant réutilisable de 0.4) sur chaque item
   de la liste. Renommer = input inline editable. Supprimer = modale de confirmation
   avec bouton rouge foncé (#8B2020). Auto-cleanup appelé dans useEffect au mount.

**Files created:** `src/components/history/session-actions.tsx`
**Files modified:** `src-tauri/src/commands/sessions.rs`, `src/components/history/session-detail.tsx`
**Depends on:** 2.2
**Validation:** Clic droit → renommer/supprimer fonctionne + sessions >60 nettoyées
