# File Tree Panel — Design Spec

**Date** : 2026-05-08
**Scope** : Panel arborescence de fichiers pour les sessions Projets

---

## Vue d'ensemble

Panel latéral d'arborescence de fichiers, accessible uniquement pour les sessions Projets. S'ouvre à droite du panel file-preview existant via un bouton dossier. Permet de naviguer dans le filesystem du projet et de prévisualiser les fichiers en lecture seule dans le panel principal.

---

## Architecture des composants

### Backend Rust (`src-tauri/src/`)

- `commands/file_tree.rs` — 2 commandes Tauri :
  - `list_directory(path, show_hidden)` → `Vec<FileEntry>`, lazy (un seul niveau), trié dossiers-first alpha insensible à la casse
  - `watch_project_directory(path)` / `unwatch_project_directory()` → file watcher `notify`, émet `file-tree-changed { path, kind }`
- `models/file_tree.rs` — struct `FileEntry { name: String, path: String, is_dir: bool, extension: Option<String> }`

**Contraintes backend** :
- Sécurité : `canonicalize()` + `starts_with(base_dir)` contre le path traversal
- Collection bornée : max 5000 entrées par appel (tronque + signale)
- Debounce watcher : 200ms
- Un seul watcher actif à la fois (stop ancien avant start nouveau)

### Frontend (`src/components/file-tree/`)

- `file-tree-panel.tsx` — aside container (slide-in, resize handle, header)
- `file-tree-node.tsx` — noeud récursif (dossier dépliant ou fichier cliquable)
- `file-tree-filter.tsx` — barre de recherche temps réel
- `file-tree-panel.css` — styles colocalisés, préfixe `ft-`

### Hooks (`src/hooks/`)

- `use-file-tree.ts` — état du tree (noeuds chargés, expanded/collapsed, filtre, width, open/close)

### Lib (`src/lib/`)

- `file-tree-icons.ts` — mapping extension → icône VS Code (réutilise la lib existante)
- `file-tree-filter.ts` — logique de filtrage (pure function, testable)
- `file-tree-hidden.ts` — liste des fichiers/dossiers masqués par défaut

Chaque fichier reste sous 200 lignes.

---

## Layout et positionnement

### Structure flex

```
<div class="agent-detail-with-preview">
  ├─ ChatView (flex: 1, minWidth: 0)
  ├─ FilePreviewPanel (panel principal existant)
  └─ FileTreePanel (nouveau, le plus à droite)
</div>
```

### Dimensions du tree panel

- Largeur par défaut : **240px**
- Min : **240px**
- Max : **500px**
- Resize handle sur le bord gauche
- Largeur persistée en localStorage par session

### Animations slide

- Ouverture : `translateX(100%)` → `translateX(0)`, `300ms cubic-bezier(0.4, 0, 0.2, 1)`
- Fermeture : inverse
- Pendant le resize : `transition: none`

### Fullscreen

- Panel principal fullscreen + arborescence ouverte → les deux occupent 100% (principal = `flex: 1`, tree = sa largeur fixe)
- Clic bouton dossier = ferme le tree (revient au principal seul en fullscreen)
- Le tree ne peut jamais être fullscreen seul

### Bouton dossier

- Position : header du panel principal (`fp-head`), à gauche du bouton fullscreen
- Même style que le bouton fullscreen existant (`.fp-head-btn`)
- Visible uniquement si `activeProject` existe
- Surbrillance quand le tree est ouvert
- Tooltip : clé i18n `fileTree.toggleTree`

---

## Data flow

1. Ouverture du tree → `list_directory(project.path)` → charge la racine
2. Dépliage d'un dossier → `list_directory(dossier.path)` → enfants stockés dans `Map<string, FileEntry[]>` (max 500 dossiers ouverts)
3. Événement `file-tree-changed` → recharge le `list_directory` du parent concerné (pas tout l'arbre)
4. Clic fichier → `onFilePreview(path)` → preview lecture seule dans le panel principal (syntax highlighting, pas de diff)
5. Filtre texte → filtrage côté frontend sur noeuds chargés (pure function, pas d'appel backend)

---

## Composants frontend détaillés

### `file-tree-panel.tsx`

- `<aside>` avec classe `ft-panel`
- Header : barre de recherche
- Body : liste scrollable des noeuds
- Resize handle bord gauche
- CSS variable `--ft-width`

### `file-tree-node.tsx`

- Dossier : chevron rotatif (0° → 90°, `200ms ease-out`), icône dossier, nom
- Fichier : icône VS Code par extension, nom
- Indentation : `padding-left: calc(depth * 16px)`
- Hover : `var(--surface-hover)`
- Fichier actif (prévisualisé) : surbrillance persistante

### Animations expand/collapse

- `max-height: 0` → `max-height: var(--ft-children-height)` (calculé via ref)
- `transition: max-height 200ms ease-out, opacity 150ms ease-out`
- Chevron en sync : `transform: rotate(90deg)`, `transition: transform 200ms ease-out`

### `file-tree-filter.tsx`

- Input contrôlé, debounce 150ms
- Match partiel insensible à la casse
- Parents auto-dépliés quand un enfant matche
- Icône "x" pour vider
- `Escape` pour vider et perdre le focus

### `file-tree-icons.ts`

- Réutilise la lib VS Code existante
- Fallback : icône fichier générique
- Dossiers : icône dossier (fermé/ouvert selon état)

---

## Fichiers masqués par défaut

Exclus de l'arborescence (côté Rust + liste frontend pour cohérence) :
- `.git`, `.DS_Store`, `.next`, `.turbo`, `__pycache__`, `dist`, `target`, `build`, `.cache`

**Conservés** : `node_modules` (visible)

---

## CSS et tokens

**Préfixe** : `ft-` (vérifier par grep l'absence de collision avant implem)

**Tokens réutilisés** :
- Background : `var(--void)`
- Bordure : `var(--edge)`
- Texte : `var(--ink)`, `var(--ink-muted)`
- Hover : `var(--surface-hover)`
- Scroll : `scrollbar-width: thin; scrollbar-color: var(--ink-faint) transparent`
- Font : `var(--font-mono)`, `var(--text-xs)`
- Barre de recherche : style `.form-input` de `global.css`
- Bouton : style `.fp-head-btn` existant
- Animations : `--ease-smooth` (200ms) pour noeuds, cubic-bezier (300ms) pour slide panel

**Nouveaux tokens** : aucun. Tout couvert par l'existant.

**Thèmes** : dark + light fonctionnent automatiquement via les tokens.

---

## i18n — 7 langues

Clé `"fileTree"` dans `en.json`, `fr.json`, `es.json`, `de.json`, `it.json`, `zh.json`, `ja.json` :

| Clé | FR | EN |
|---|---|---|
| `filterPlaceholder` | Filtrer les fichiers... | Filter files... |
| `emptyDirectory` | Ce dossier est vide | This folder is empty |
| `tooManyEntries` | Trop d'éléments ({{count}} max) | Too many entries ({{count}} max) |
| `toggleTree` | Arborescence du projet | Project file tree |
| `loadError` | Impossible de lire ce dossier | Could not read this folder |
| `noResults` | Aucun résultat | No results |

ES, DE, IT, ZH, JA : traductions équivalentes (voir implem).

---

## Tests

### Backend Rust

`commands/file_tree_test.rs` :
- Tri correct (dossiers-first, alpha)
- Exclusion des dossiers masqués (`.git` exclu, `node_modules` conservé)
- Path traversal bloqué
- Dossier inexistant → erreur propre
- Borne 5000 entrées respectée

`models/file_tree_test.rs` :
- Sérialisation/désérialisation `FileEntry`
- Extraction extension (cas `.tar.gz`, pas d'extension, fichier `.dotfile`)

### Frontend

`src/lib/file-tree-filter.test.ts` :
- Match partiel insensible à la casse
- Chaîne vide → tout retourné
- Parents auto-dépliés pour fichiers matchés
- Caractères spéciaux (accents, espaces)

`src/lib/file-tree-hidden.test.ts` :
- `.git` masqué, `node_modules` visible
- `.DS_Store` masqué, `.env` visible

`src/lib/file-tree-icons.test.ts` :
- Extensions connues → bonne icône
- Extension inconnue → icône générique
- Dossier → icône dossier

Pas de tests composants React (visuels, dépendent du runtime Tauri).

---

## Contraintes non-fonctionnelles

- Zéro flickering UI : tout async, chargement lazy, état local stable
- Mise à jour temps réel sans refresh (file watcher + événements Tauri)
- Aucune valeur CSS hardcodée
- Aucun texte en dur (tout via i18n)
- Fichiers < 200 lignes
