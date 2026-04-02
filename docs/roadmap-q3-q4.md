# CL-GO-DASH — Roadmap Quest 3 + Quest 4

## Quest 3 — Personnalité

### Mission 3.1 — Backend lecture markdown + MWeb opener

**Context:** Les fichiers personnalité sont dans `~/.local/share/cl-go/memory/core/` :
identity.md, me.md, principles.md, note-to-self.md, user.md. Plus `notes.md` dans
`~/.local/share/cl-go/inbox/` et `idea-discovery.md` dans le même dossier.
MWeb ouvre un fichier via `open -a MWeb /path/to/file.md` (testé, fonctionne).
Si MWeb n'est pas installé, fallback vers `open /path/to/file.md` (éditeur par défaut).

**Objective:** Commandes pour lister et lire les fichiers personnalité + ouvrir dans MWeb.

**Tasks:**
1. `commands/personality.rs` : commande `list_personality_files()` qui retourne la liste
   des fichiers avec nom, path, description courte (extraite de la première ligne h1/h2).
   Commande `read_personality_file(filename)` qui retourne le contenu brut markdown.
2. Commande `open_in_editor(path)` qui tente `open -a MWeb {path}`, fallback `open {path}`
   si MWeb absent. Vérifier l'existence du fichier avant d'ouvrir. Chemin validé par
   regex (doit commencer par la racine mémoire, pas de `..`).

**Files created:** `src-tauri/src/commands/personality.rs`
**Files modified:** `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
**Depends on:** 0.2
**Validation:** `cargo check` + invoke `list_personality_files` retourne la liste

---

### Mission 3.2 — Frontend liste + vue markdown

**Context:** Design mockup : liste de fichiers dans le list-panel (icône 📄 + nom + description).
Détail : rendu markdown propre avec style `.md-view` (h1, h2, p, strong en couleur pulse,
blockquote avec barre accent). Bouton discret "↗ MWeb" dans le header pour ouvrir
le fichier dans l'éditeur externe.

**Objective:** Onglet Personnalité complet.

**Tasks:**
1. `personality-list.tsx` : liste des fichiers. Chaque item : icône + nom du fichier +
   description. Clic = charge le contenu dans le détail.
2. `markdown-viewer.tsx` : composant qui prend du markdown brut et le rend en HTML stylé.
   Utiliser `react-markdown` (ou parser léger). Appliquer les classes `.md-view` du design.
   `src/services/personality.ts` : fonctions invoke pour list + read + open.
3. Intégrer le bouton "↗ MWeb" dans le header du détail. Appelle `open_in_editor(path)`.

**Files created:** `src/components/personality/personality-list.tsx`,
`src/components/personality/markdown-viewer.tsx`, `src/services/personality.ts`
**Files modified:** —
**Depends on:** 0.4, 3.1
**Validation:** `npx tsc --noEmit` + fichiers affichés, bouton MWeb ouvre le fichier

---

## Quest 4 — Polish & Hardening

### Mission 4.1 — Empty states + loading skeletons

**Context:** Chaque vue peut être vide (aucun réveil, aucune session, aucun fichier).
Les données mettent un temps à charger (surtout les JSONL volumineux).
Convention mockup : skeletons rectangulaires avec shimmer, messages "Aucun réveil programmé"
+ CTA dans les empty states.

**Objective:** Aucune zone vide sans message, aucun chargement sans feedback visuel.

**Tasks:**
1. `skeleton.tsx` : composant réutilisable avec shimmer CSS (animation gradient).
   Props : width, height, count (nombre de lignes). Utilisé dans les listes.
2. `empty-state.tsx` : composant réutilisable avec message + CTA optionnel.
   3 variantes prédéfinies : "Aucun réveil", "Pas d'historique", "Aucun fichier".
   Intégrer dans heartbeat-list, history-list, personality-list.

**Files created:** `src/components/ui/skeleton.tsx`, `src/components/ui/empty-state.tsx`
**Files modified:** composants list de chaque onglet (heartbeat-list, history-list, personality-list)
**Depends on:** 1.2, 2.2, 3.2
**Validation:** Vider les données → empty states visibles. Slow network → skeletons visibles.

---

### Mission 4.2 — Error handling + toast notifications

**Context:** Les erreurs IPC (fichier introuvable, permission refusée, JSONL corrompu)
doivent être gérées proprement. Fail CLOSED : une erreur bloque, elle ne laisse pas passer.
Messages génériques côté UI (pas de stack trace). Toasts pour les actions (supprimé, renommé).

**Objective:** Gestion d'erreurs cohérente dans toute l'app.

**Tasks:**
1. `error-boundary.tsx` : React error boundary global. Catch les erreurs de rendu,
   affiche un message propre. `toast.tsx` : composant toast slide-in depuis le haut,
   auto-dismiss 3s. Variantes : success (signal-ok), error (signal-error), info (pulse).
2. Wrapper les appels `invoke()` dans un try/catch dans chaque service.
   Afficher un toast en cas d'erreur. Logger les erreurs en console (sans données sensibles).

**Files created:** `src/components/ui/error-boundary.tsx`, `src/components/ui/toast.tsx`
**Files modified:** `src/App.tsx`, services (heartbeat.ts, sessions.ts, personality.ts)
**Depends on:** 4.1
**Validation:** Simuler une erreur IPC → toast visible, pas de crash

---

### Mission 4.3 — Build final + smoke test

**Context:** Toutes les features sont implémentées. Il faut builder l'app Tauri en mode
release, vérifier que tout fonctionne sur macOS, et corriger les derniers bugs.

**Objective:** App buildée et fonctionnelle.

**Tasks:**
1. `npm run tauri build` — vérifier que le build passe sans erreur.
   Tester manuellement : les 3 onglets, dark/light, clic droit, Escape/Enter,
   run heartbeat, lecture sessions, ouverture MWeb.
2. Corriger les bugs trouvés pendant le smoke test.

**Files created:** —
**Files modified:** selon les bugs trouvés
**Depends on:** 4.2
**Validation:** `npm run tauri build` exit 0 + smoke test manuel OK

---

## Ordre d'exécution

```
Quest 0 (fondations) — séquentiel
  └─ 0.1 Scaffolding ────────── aucune dépendance
  └─ 0.2 Types + Config ─────── dépend de 0.1
  └─ 0.3 Design + Layout ────── dépend de 0.1 (parallèle avec 0.2)
  └─ 0.4 Hooks réutilisables ── dépend de 0.3

Quest 1 (Heartbeat) ─────────── parallélisable avec Q2 et Q3 après Q0
  └─ 1.1 Backend CRUD ─────────── dépend de 0.2
  └─ 1.2 Frontend ─────────────── dépend de 0.4 + 1.1
  └─ 1.3 Run + Voyants ────────── dépend de 1.2
  └─ 1.4 Warnings ─────────────── dépend de 1.2 (parallèle avec 1.3)

Quest 2 (Historique) ─────────── parallélisable avec Q1 et Q3 après Q0
  └─ 2.1 Backend parser ───────── dépend de 0.2
  └─ 2.2 Frontend liste ───────── dépend de 0.4 + 2.1
  └─ 2.3 Vue conversation ─────── dépend de 2.2
  └─ 2.4 Rename/Delete ────────── dépend de 2.2 (parallèle avec 2.3)

Quest 3 (Personnalité) ──────── parallélisable avec Q1 et Q2 après Q0
  └─ 3.1 Backend ──────────────── dépend de 0.2
  └─ 3.2 Frontend ─────────────── dépend de 0.4 + 3.1

Quest 4 (Polish) ─────────────── après Q1 + Q2 + Q3
  └─ 4.1 Empty states ────────── dépend de 1.2 + 2.2 + 3.2
  └─ 4.2 Error handling ──────── dépend de 4.1
  └─ 4.3 Build final ─────────── dépend de 4.2
```

## Checklist sécurité

- [ ] Pas de `==` pour comparer des secrets (pas applicable ici)
- [ ] Validation regex sur les arguments shell (crontab, open)
- [ ] Chemins filesystem validés (pas de `..`, racine whitelist)
- [ ] Messages d'erreur génériques côté UI (pas de stack trace)
- [ ] Pas de secrets dans les logs (pas applicable ici)
- [ ] Écriture atomique config.json (tmp + rename)
- [ ] Permissions Tauri minimales (fs:read limité aux dossiers nécessaires)
