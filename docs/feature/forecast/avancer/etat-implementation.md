# État réel de l'implémentation Forecast — Mai 2026

## Résumé

L'implémentation du module forecast est **non fonctionnelle**. La structure de fichiers existe (backend Rust + frontend React) mais presque rien n'est câblé de bout en bout. Aucun flux utilisateur ne fonctionne complètement.

**Score review GPT-5.5 : 3/10**

---

## Ce qui existe (structure posée)

### Backend Rust — `src-tauri/src/services/forecast/`
- `mod.rs` — déclarations de sous-modules
- `types.rs` — structs ForecastRequest, ForecastResult, Quantiles, Annotation, Scenario, ModelConfig, etc.
- `catalog.rs` — catalogue statique : 4 modèles Chronos-Bolt (tiny/mini/small/base) + 3 TimeGPT-2 (mini/standard/pro)
- `storage.rs` — CRUD analyses (save/load/delete/list) avec écriture atomique tmp+rename, validation UUID
- `client_nixtla.rs` — client HTTP pour l'API Nixtla TimeGPT-2
- `client_chronos.rs` — client HTTP pour le sidecar Chronos local
- `sidecar.rs` — lifecycle sidecar Python (start/stop/health/kill_orphan)
- `model_manager/mod.rs` — is_installed, installed_models, install, uninstall
- `model_manager/download.rs` — download depuis HuggingFace avec progress channel

### Agent Tools — `src-tauri/src/services/agent_local/`
- `tool_definitions_forecast.rs` — 3 tools définis (forecast, forecast_analyze, forecast_read)
- `tool_dispatcher_forecast.rs` — dispatch vers les clients, gestion file_path
- Injection dans `tool_definitions.rs`, `tool_dispatcher.rs`, `tool_validate.rs`, `permission_gate.rs`

### Commandes Tauri — `src-tauri/src/commands/forecast.rs`
- `run_forecast` — lance un forecast (créée après review, minimale)
- `list_forecast_analyses` — liste les analyses
- `get_forecast_analysis` — charge une analyse
- `delete_forecast_analysis` — supprime une analyse
- `list_forecast_models` — modèles installés + disponibles
- `install_forecast_model` — download HuggingFace
- `uninstall_forecast_model` — supprime les fichiers
- `list_forecast_providers_catalog` — catalogue providers

### Frontend React — `src/components/forecast/`
- `forecast-panel.tsx` — container principal avec router de sections
- `forecast-header.tsx` — header avec nav trigger + fullscreen
- `forecast-nav.tsx` — accordion dépliable avec 5 sections
- `forecast-empty.tsx` — état vide avec boutons (import câblé, coller/URL non câblés)
- `forecast-config.tsx` — écran de configuration (JAMAIS RENDU, composant mort)
- `sections/forecast-view.tsx` — KPIs + mini chart SVG + tableau (pas d'ECharts)
- `sections/forecast-scenarios.tsx` — placeholder "coming soon"
- `sections/forecast-analysis.tsx` — placeholder "coming soon"
- `sections/forecast-notes.tsx` — liste d'annotations depuis le backend
- `sections/forecast-history.tsx` — liste avec recherche
- `widgets/export-dropdown.tsx` — dropdown 7 formats (rendu dans le footer, `onExport` = console.log)
- `model-browser/forecast-models.tsx` — browser de modèles
- `model-browser/model-card.tsx` — card avec specs
- `model-browser/model-install-btn.tsx` — bouton install avec progress bar
- `model-browser/model-specs.tsx` — fiche technique + désinstallation

### Infrastructure
- `hooks/use-forecast-panel.ts` — state management panel (section, nav, analysisId)
- `mode-selector.tsx` — dropdown portal pour basculer File Preview / Forecast
- Slide transition CSS dans `file-preview-panel.css` (translateX entre les deux modes)
- Câblage : agent-local-tab → TabBar → AgentChatDetail → FilePreviewPanel
- Onglet "Forecast" dans settings-tab.tsx avec icône ChartLineUp
- Tokens CSS `--fc-*` dans dark.css + light.css (16 tokens)
- i18n : clés `forecast.*` dans les 7 langues

---

## Ce qui NE FONCTIONNE PAS (liste exhaustive)

### Flux principal — Lancer un forecast
1. **Import de fichier** : le file picker s'ouvre mais le fichier sélectionné n'est pas traité (pas de parsing CSV/Excel, pas de passage vers la config)
2. **Coller des données** : bouton sans handler (TODO dans le code)
3. **Depuis une URL** : bouton sans handler (TODO dans le code)
4. **Config screen** : `forecast-config.tsx` existe mais n'est JAMAIS importé ni rendu — le flux import → config → lancement n'existe pas
5. **`run_forecast`** : la commande Tauri existe mais n'est jamais appelée depuis le frontend
6. **Résultats** : impossible d'en voir car impossible de lancer un forecast

### Téléchargement de modèles
1. **Progress bar ne progresse pas** : le download démarre peut-être mais l'UI ne reflète pas l'avancement correctement
2. **Validation silencieuse** : le bouton passe à "Installé" sans vérifier que le téléchargement a réellement abouti
3. **Aucune vérification d'intégrité** : SHA256 supprimé, pas de check que les fichiers sont complets
4. **Dossier d'installation** : `~/.local/share/cl-go-dash/forecast-models/{model-name}/` — mais le sidecar Python qui les utilise n'existe pas

### Clés API TimeGPT-2
1. **Aucune UI pour ajouter la clé Nixtla** : le backend supporte `set_api_key("nixtla", key)` et `test_key("nixtla")` mais il n'y a aucun formulaire/modal dans le frontend pour que l'utilisateur entre sa clé
2. **Nixtla n'apparaît pas dans les connecteurs** : il faudrait l'ajouter dans le catalogue des connecteurs ou créer une section dédiée dans l'onglet Forecast des settings

### Sidecar Python Chronos-Bolt
1. **Le script `server.py` n'existe pas** : `sidecar.rs` cherche `~/.local/share/cl-go-dash/forecast-sidecar/server.py` mais ce fichier n'a jamais été créé
2. **Pas de requirements.txt** : même si le script existait, les dépendances Python (chronos, torch, fastapi, uvicorn) ne sont pas gérées
3. **ChronosSidecar est dans l'état Tauri** mais `start()` n'est jamais appelé automatiquement
4. **Aucune commande Tauri pour démarrer/arrêter le sidecar** depuis le frontend

### Exports
1. **`onExport` = `console.log`** : le dropdown s'affiche mais ne fait rien
2. **`export.rs` n'existe pas** : aucun backend d'export (CSV, Excel, PDF, etc.)
3. **ECharts non installé** : `echarts` et `echarts-for-react` ne sont pas dans `package.json`
4. **PNG/SVG depuis ECharts** : impossible sans ECharts

### Sections du panel
1. **Vue principale** : mini chart SVG basique (polyline), pas d'ECharts, pas de bandes de confiance, pas de zoom, pas de tooltip
2. **Scénarios** : placeholder "coming soon"
3. **Analyse** : placeholder "coming soon"
4. **Notes** : affiche les annotations du backend mais pas d'input pour en créer manuellement
5. **Historique** : liste les analyses mais pas de suppression ni comparaison

---

## Rapport review GPT-5.5 (complet)

### Score : 3/10 — 4 critiques, 12 majeurs, 8 mineurs

### Critiques trouvés (corrigés par Sonnet après review)
1. **Path traversal dans storage.rs** — id concaténé sans validation → CORRIGÉ (validate_analysis_id avec regex UUID)
2. **`run_forecast` inexistant** → CORRIGÉ (commande créée, minimale)
3. **Sidecar jamais injecté dans l'état Tauri** → CORRIGÉ (.manage(ChronosSidecar::new()))
4. **Race condition sidecar** → CORRIGÉ (PID check avant port scan)

### Majeurs trouvés
1. **Validation incomplète** — confidence_level manquant → CORRIGÉ
2. **file_path accepté mais non géré** → CORRIGÉ (lecture fichier dans dispatcher)
3. **Hook ne reset pas quand sessionId change** → PARTIELLEMENT CORRIGÉ (Sonnet a eu des problèmes ESLint)
4. **Pas d'ECharts** — chart SVG basique → NON CORRIGÉ
5. **SHA256 calculé mais ignoré** → CORRIGÉ (supprimé)
6. **Annulation download = flag local** → NON CORRIGÉ
7. **installed_models() inclut .cache** → CORRIGÉ
8. **stderr sidecar jamais lu** → CORRIGÉ (Stdio::null)
9. **Erreurs brutes exposées** → CORRIGÉ (messages génériques)
10. **Collections non bornées** → CORRIGÉ (MAX_ANALYSES=500)
11. **Sections placeholder** → NON CORRIGÉ
12. **Boutons état vide sans handler** → PARTIELLEMENT CORRIGÉ (import OK, coller/URL = TODO)

### Mineurs trouvés
1. Division par zéro ChartPreview → CORRIGÉ
2. Textes hardcodés → CORRIGÉ (i18n)
3. mode-selector.css manquant → CORRIGÉ (inline styles)
4. Pas de résumé tool bubble forecast → NON CORRIGÉ
5. Fichiers > 200 lignes → NON VÉRIFIÉ
6. Token --tool-forecast vs --fc-tool → clarification nécessaire
7. toLocaleDateString sans locale → NON CORRIGÉ

---

## Dépendances manquantes

### package.json
- `echarts` — bibliothèque de graphiques
- `echarts-for-react` — wrapper React pour ECharts

### Cargo.toml
- `rust_xlsxwriter` — export Excel
- `genpdf` ou `printpdf` — export PDF

### Sidecar Python
- Script `server.py` (FastAPI) qui charge Chronos-Bolt et expose `/predict` + `/health`
- `requirements.txt` avec : `chronos-forecasting`, `torch`, `fastapi`, `uvicorn`
- À packager dans `src-tauri/resources/forecast-sidecar/`

---

## Architecture des fichiers

```
src-tauri/src/
├── commands/forecast.rs              # Commandes Tauri IPC
├── services/forecast/
│   ├── mod.rs                        # Déclarations modules
│   ├── types.rs                      # Types Rust (Request, Result, etc.)
│   ├── catalog.rs                    # Catalogue modèles statique
│   ├── storage.rs                    # CRUD analyses persistées
│   ├── client_nixtla.rs              # Client HTTP Nixtla
│   ├── client_chronos.rs             # Client HTTP sidecar local
│   ├── sidecar.rs                    # Lifecycle sidecar Python
│   └── model_manager/
│       ├── mod.rs                    # Install/uninstall/list
│       └── download.rs              # Download HuggingFace
├── services/agent_local/
│   ├── tool_definitions_forecast.rs  # Définitions 3 tools
│   └── tool_dispatcher_forecast.rs   # Dispatch vers clients

src/
├── hooks/use-forecast-panel.ts       # State management panel
├── components/agent-local/
│   └── mode-selector.tsx             # Dropdown portal File Preview/Forecast
├── components/forecast/
│   ├── forecast-panel.tsx            # Container principal
│   ├── forecast-header.tsx           # Header + nav trigger + fullscreen
│   ├── forecast-nav.tsx              # Accordion navigation
│   ├── forecast-empty.tsx            # État vide + boutons
│   ├── forecast-config.tsx           # Config screen (MORT — jamais rendu)
│   ├── forecast-panel.css            # Styles panel
│   ├── forecast-empty.css            # Styles empty state
│   ├── forecast-view.css             # Styles vue principale
│   ├── forecast-config.css           # Styles config (MORT)
│   ├── forecast-sections.css         # Styles sections partagés
│   ├── forecast-history.css          # Styles historique
│   ├── sections/
│   │   ├── forecast-view.tsx         # KPIs + mini chart SVG + tableau
│   │   ├── forecast-scenarios.tsx    # Placeholder
│   │   ├── forecast-analysis.tsx     # Placeholder
│   │   ├── forecast-notes.tsx        # Timeline annotations
│   │   └── forecast-history.tsx      # Liste + recherche
│   ├── widgets/
│   │   ├── export-dropdown.tsx       # Dropdown 7 formats (console.log)
│   │   └── export-dropdown.css
│   └── model-browser/
│       ├── forecast-models.tsx       # Browser modèles
│       ├── forecast-models.css       # Styles browser
│       ├── model-card.tsx            # Card modèle
│       ├── model-install-btn.tsx     # Bouton install + progress
│       ├── model-specs.tsx           # Fiche technique
│       └── model-specs.css           # Styles specs
```

---

## Plan d'implémentation original

Le plan complet est dans `/Users/kevinh/.claude/plans/kind-sparking-rivest.md` (10 phases).

Les docs de design/recherche sont dans `/Users/kevinh/Projects/CL-GO-DASH/docs/feature/forecast/` :
- `01-research-context.md` à `12-design-model-management.md`

---

## Pour la reprise — priorités

1. **Créer le sidecar Python** (`server.py` + requirements) — sans ça, aucun forecast local ne peut tourner
2. **Câbler le flux complet** : import → config → run_forecast → afficher résultats
3. **Installer ECharts** et remplacer le mini chart SVG par un vrai graphique interactif
4. **Créer l'UI pour la clé API Nixtla** dans l'onglet Forecast des settings
5. **Corriger le téléchargement de modèles** — vérification réelle, feedback utilisateur fiable
6. **Implémenter les exports** — backend `export.rs` + câblage frontend
7. **Compléter les sections** — scénarios, analyse, annotations manuelles
8. **Tests** — zéro test actuellement

---

## Erreurs commises pendant l'implémentation

1. Déclaration prématurée de "tout est terminé" alors que rien n'était câblé de bout en bout
2. Création de composants sans les connecter dans le rendu
3. Pas de test fonctionnel (npm run tauri dev + utilisation réelle) avant de déclarer la fin
4. Review GPT-5.5 lancée avec le mauvais modèle (Opus 4.7 au lieu de Codex GPT-5.5 → 20€ perdus)
5. Corrections post-review partielles — Sonnet bloqué sur un problème ESLint sans finir le reste
6. Aucun sidecar Python créé — le cœur du fonctionnement local manque totalement
