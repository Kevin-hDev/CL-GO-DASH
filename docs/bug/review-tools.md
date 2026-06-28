# Review de sécurité — Tools de l'agent IA

> **Périmètre** : tous les tools exposés à l'IA dans `src-tauri/src/services/agent_local/`, à l'exclusion des tools forecast.
> **Mode** : review read-only. Aucune modification apportée au code.
> **Date** : 2026-06-28
> **Auteur** : analyse automatisée + contre-vérification manuelle des points critiques.

---

## Sommaire

1. [Architecture & points forts](#1-architecture--points-forts)
2. [Problèmes critiques](#2-problèmes-critiques)
3. [Problèmes haute sévérité](#3-problèmes-haute-sévérité)
4. [Problèmes moyenne sévérité](#4-problèmes-moyenne-sévérité)
5. [Problèmes basse sévérité](#5-problèmes-basse-sévérité)
6. [Plan de correction recommandé](#6-plan-de-correction-recommandé)

---

## 1. Architecture & points forts

### Organisation

Les tools sont répartis en 5 catégories :

| Catégorie | Fichiers clés |
|-----------|---------------|
| **Système/Fichiers** | `tool_bash.rs`, `tool_bash_background.rs`, `tool_bash_long.rs`, `tool_files.rs`, `tool_glob.rs`, `tool_grep.rs`, `tool_scan_timeout.rs` |
| **Documents/Office** | `tool_document_read.rs`, `tool_document_write*.rs`, `tool_spreadsheet_*.rs`, `tool_image_*.rs`, `tool_office_utils.rs` |
| **Web** | `tool_web_fetch.rs`, `tool_web_fetch_ip.rs`, `tool_web_search.rs` |
| **Interactif/Plan/Todo** | `tool_interactive*.rs`, `tool_plan*.rs`, `tool_todo*.rs` |
| **Infrastructure** | `tool_dispatcher*.rs`, `tool_executor*.rs`, `tool_delegate.rs`, `tool_hooks.rs`, `tool_validate.rs`, `tool_result_budget.rs` |

### Points forts confirmés (à conserver)

- **SSRF `web_fetch` très solide** (`tool_web_fetch.rs`) : validation `ssrf::validate_url` à **chaque hop** de redirection, IP épinglée au client via `pinned_client` (pas de TOCTOU DNS rebinding), `allow_private=false` en production, bornes strictes (`MAX_BODY_BYTES=5 MiB`, `MAX_REDIRECTS=3`, `TIMEOUT=15s`).
- **Path traversal bloqué** partout via `security::validate_read_path` / `validate_write_path` (canonicalisation + `starts_with` sur `working_dir` ou `allowed_read_roots`).
- **Whitelists d'extensions** strictes sur tous les tools office/image.
- **Plan mode verrouillé** : `exit_plan` exige une validation utilisateur (`tool_plan_approval.rs:49`), impossible d'auto-approuver. La classification se base uniquement sur les `selected_ids` stables (pas sur les labels francophones manipulables).
- **Sous-agents correctement isolés au runtime** (vérifié manuellement) :
  - `explorer` reçoit une liste restreinte de tools via `subagent_task.rs:80` (`get_explorer_tool_definitions`).
  - `coder` obtient un worktree git isolé via `subagent_working_dir.rs:9-11`, créé au runtime.
  - `bash_touches_sensitive_data` **EST branchée** dans `tool_executor_write.rs:88-94` via `must_prompt_for_sensitive_bash`.
- **Todo/Plan storage** : `validate_session_id` regex `^[a-f0-9\-]+$` max 64 chars → path traversal bloqué.
- **Génération XML DOCX** via `quick_xml::Writer` avec échappement automatique → pas d'injection XML en écriture.
- **Sanitization sortie MCP** (`tool_mcp.rs:162`) : strip des control chars + bidi overrides → défense contre les injections homoglyphes.
- **Budget de tokens** (`tool_result_budget.rs`) : préserve les 2 derniers résultats, remplace les plus anciens par un placeholder.

### Note sur le mode `auto`

Le mode `auto`/`subagent` est **volontairement un mode bypass permissions** (full access), comme tous les CLI agents du marché. L'utilisateur est conscient des risques et peut rester en mode `manuel` s'il n'a pas confiance. Le point 3.6 ci-dessous documente ce comportement à titre informatif, **ce n'est pas un bug**.

---

## Suivi des corrections — 2026-06-28

Statuts utilisés :

- `Corrigé` : point repris et corrigé.
- `Corrigé avec nuance` : risque réel, mais corrigé sans reprendre une partie trop large ou inexacte du finding.
- `À ne pas reprendre / faux positif` : point jugé incorrect tel qu'écrit, ou déjà couvert par le code existant.

| Point | Statut | Correctif appliqué | Fichiers principaux | Tests ajoutés / mis à jour | Justification / nuance |
|-------|--------|--------------------|---------------------|-----------------------------|------------------------|
| 2.1 DOCX zip bomb | Corrigé | Limite taille source, inspection zip, limite XML décompressé, ratio zip, limite texte extrait, erreur explicite sur XML malformé. | `tool_office_limits.rs`, `tool_document_read.rs` | `tool_document_read_tests.rs` | Le DOCX n'est plus lu sans garde en mémoire. |
| 2.2 XLSX/XLSM/ODS zip bomb | Corrigé | Taille source bornée et inspection zip avant ouverture `calamine` / `umya`. | `tool_office_limits.rs`, `tool_spreadsheet_calamine.rs`, `tool_spreadsheet_write_edit.rs` | `tool_spreadsheet_write_tests.rs` | La lecture et l'édition refusent les archives trop grosses ou suspectes avant parsing. |
| 2.3 Image resize / decode non borné | Corrigé | Taille fichier, dimensions avant décodage, pixels max, dimensions resize/crop bornées. | `tool_image_process.rs` | `tool_image_process_limits_tests.rs` | Les dimensions trop grandes échouent avant allocation lourde. |
| 2.4 CSV lu entièrement | Corrigé | Lecture CSV/TSV via `BufReader` et taille source bornée. | `tool_spreadsheet_read.rs` | Couvert par validation Rust existante + tests spreadsheet ciblés | Le fichier n'est plus chargé en une seule chaîne avant `max_rows`. |
| 2.5 Blocklist destructive | Corrigé avec nuance | Normalisation des espaces et ajout de patterns `find -delete`, `rsync --delete`, `dd of=/dev/`, `rm -rf ~/$HOME`. | `security.rs`, `tool_bash_long.rs` | `security_tests.rs` | Les exemples `dd if=`, `mkfs.` et redirections `/dev/sd` étaient déjà partiellement couverts. Le finding global reste valide car une blocklist n'est jamais exhaustive. |
| 3.1 TOCTOU `write_file` | Corrigé | Validation avant création des dossiers, refus symlink, écriture sans suivre les symlinks quand possible. | `tool_files.rs` | `tool_files_tests.rs` | Réduit la fenêtre TOCTOU et évite la création de dossiers avant validation finale. |
| 3.2 ReDoS grep | À ne pas reprendre / faux positif | Correction limitée au risque de scan long / timeout coopératif. | `tool_scan_timeout.rs`, `tool_grep.rs` | Validation Rust globale | Ne pas reprendre comme ReDoS exponentiel : `grep-regex` s'appuie sur `regex-automata`, pas un moteur backtracking classique. |
| 3.3 Timeout batch parallèle | Corrigé | Ajout d'un timeout global et écoute de l'annulation pendant le batch. | `tool_executor_parallel_batch.rs` | Validation Rust globale | Les reads parallèles ne peuvent plus bloquer tout le batch indéfiniment. |
| 3.4 MCP sans timeout | Corrigé | Timeout de 60 secondes autour des appels MCP. | `tool_mcp.rs` | Validation Rust globale | Un connecteur MCP bloqué retourne maintenant une erreur. |
| 3.5 `spawn_blocking` après timeout | Corrigé avec nuance | Conservation du `JoinHandle`, signal d'annulation et `abort()` au timeout. | `tool_scan_timeout.rs` | Validation Rust globale | Ne pas promettre que `abort()` tue un thread `spawn_blocking` déjà lancé. La correction limite l'attente et signale l'annulation, mais Rust ne force pas l'arrêt d'un travail bloquant déjà en cours. |
| 4.1 Redaction read/grep/glob/list_dir | Corrigé | Post-hook de redaction étendu aux outils de lecture et patterns PEM/SSH renforcés. | `tool_hooks.rs`, `sensitive_data.rs` | `tool_hooks_tests.rs` | Les secrets évidents sont filtrés avant retour au modèle. |
| 4.2 Pré-hook `..` | Corrigé avec nuance | Blocage de `..` sur les champs chemin structurés. Pas de blocage global dans bash. | `tool_hooks.rs` | Validation Rust globale | Ne pas bloquer `bash` contenant `..` : `cd ..`, `ls ..`, `git diff ..` peuvent être légitimes. |
| 4.3 `interactive_choice_gate` global | Corrigé | `session_id` ajouté à l'event, à la commande Tauri et à la vérification backend. | `interactive_choice_gate.rs`, `tool_interactive.rs`, `types_stream.rs`, `agent_settings.rs`, `interactive-choice-panel.tsx`, `agent-interactive.ts` | `tool_interactive_tests.rs`, `interactive-choice-panel.test.tsx`, tests hooks TS | Une réponse d'une autre session est refusée et la demande reste en attente. |
| 4.4 Plan guard race | Corrigé | Re-vérification Plan Mode juste avant les mutations. | `tool_executor_write.rs`, `tool_executor_sequential.rs` | Validation Rust globale | La garde est répétée au dernier moment avant dispatch effectif. |
| 4.5 WriteGuard office/image | Corrigé | `WriteGuard` étendu à `write_spreadsheet`, `write_document`, `process_image`; lectures office/image enregistrées. | `tool_executor_helpers.rs` | Validation Rust globale | La règle read-before-write est maintenant cohérente sur office/image. |
| 4.6 XML DOCX malformé | Corrigé | Parser XML renvoie une erreur explicite au lieu d'un texte partiel silencieux. | `tool_document_read.rs` | `tool_document_read_tests.rs` | Les documents corrompus ne sont plus présentés comme lecture valide. |
| 4.7 Formula injection XLSX | Corrigé | `set_cell` / `set_row` écrivent du texte sûr; seules les opérations `set_formula` écrivent une formule. | `tool_spreadsheet_write_edit.rs`, `tool_spreadsheet_write_new.rs` | `tool_spreadsheet_write_tests.rs` | Les chaînes commençant par `=`, `+`, `-`, `@` ne deviennent plus des formules par défaut. |
| 4.8 Log office non rotatif | Corrigé | Rotation par taille et args redacts avant écriture du log. | `tool_dispatcher_office.rs` | Validation Rust globale | Le log ne grossit plus sans borne et expose moins d'arguments sensibles. |
| 4.9 Budget contournable via persist | Corrigé avec nuance | Le chemin absolu n'est plus exposé; la persistance utile reste. | `tool_dispatcher.rs`, `tool_result_truncate.rs` | Validation Rust globale | À relire comme limite de conception : la persistance des gros résultats est voulue, mais l'app ne doit pas divulguer le chemin absolu local. |
| 4.10 `execute_background_shell` | Corrigé | Validation destructive ajoutée dans la fonction elle-même. | `tool_bash_long.rs` | `security_tests.rs` | Un appel futur direct ne contourne plus la vérification destructive. |
| 5.1 Chemin absolu filesystem | Corrigé | Preview des gros résultats basée sur un identifiant relatif non sensible. | `tool_dispatcher.rs`, `tool_result_truncate.rs` | Validation Rust globale | Le modèle ne reçoit plus `~/.local/share/...` en clair. |
| 5.2 `repair_json` | Corrigé | Réparation heuristique supprimée; JSON invalide = erreur. | `tool_spreadsheet_write.rs` | `tool_spreadsheet_write_tests.rs` | Échec fermé plutôt que correction ambiguë. |
| 5.3 Redaction incomplète | Corrigé | Ajout redaction PEM / clés privées / certificats. | `sensitive_data.rs` | `tool_hooks_tests.rs` | Les blocs sensibles classiques sont masqués. |
| 5.4 `is_ip_private` | Corrigé | Alignement sur `ssrf::is_blocked_ip`. | `tool_web_fetch_ip.rs` | Tests SSRF/link-preview existants | Un seul référentiel de blocage IP. |
| 5.5 `host.starts_with("fd")` | Corrigé | Règle affinée pour ne viser les littéraux IPv6 que si le host contient `:`. | `ssrf.rs`, `link_preview/security.rs` | Tests SSRF/link-preview existants | Réduit les faux positifs sur domaines légitimes. |
| 5.6 `normalize_formula` | Corrigé | Remplacement fonctions/séparateurs seulement hors chaînes de texte. | `tool_office_utils.rs` | Validation Rust globale | Les textes entre guillemets ne sont plus modifiés. |
| 5.7 WebP quality | Corrigé | Avertissement clair si `quality` est ignorée pour WebP lossless. | `tool_image_process.rs` | `tool_image_process_limits_tests.rs` | Le résultat indique la limite au lieu de rester silencieux. |
| 5.8 Conversions bornées | Corrigé | `u64 -> u32/u16` remplacé par conversions vérifiées. | `tool_office_utils.rs`, `tool_spreadsheet_write*.rs`, `tool_image_process.rs` | `tool_spreadsheet_write_tests.rs`, `tool_image_process_limits_tests.rs` | Les valeurs trop grandes déclenchent une erreur. |
| 5.9 Erreurs silencieuses / ops inconnues | Corrigé | Erreurs agrégées dans `list_dir`/`glob`/`grep`; opérations inconnues refusées. | `tool_files.rs`, `tool_glob.rs`, `tool_grep.rs`, `tool_document_write_xml.rs`, `tool_spreadsheet_write*.rs` | `tool_spreadsheet_write_tests.rs` | Les erreurs ne disparaissent plus silencieusement. |
| 5.10 `extract_persist_path` fragile | Corrigé avec nuance | Le chemin absolu n'est plus exposé; le placeholder utilise un identifiant relatif. | `tool_dispatcher.rs`, `tool_result_truncate.rs` | Validation Rust globale | Le parsing fragile devient moins sensible car le texte ne contient plus de chemin absolu à préserver. |
| 5.11 Skill loader | À ne pas reprendre / faux positif | Aucun correctif de traversal nécessaire. | Aucun | Tests existants | Faux positif : le nom demandé est déjà rejeté s'il contient `..`, `/` ou `\`. Le nom parsé local ne permet pas de traverser le filesystem. |
| 5.12 Bash cwd update | Corrigé | Update du `cwd` attendu inline et erreur propagée dans le résultat. | `tool_dispatcher.rs` | Validation Rust globale | Plus de `tokio::spawn` silencieux pour une mutation d'état session. |

---

## 2. Problèmes critiques

### 2.1 Aucune protection zip bomb — DOCX en lecture

**Fichier** : `tool_document_read.rs:61-68`

```rust
let mut archive = ZipArchive::new(Cursor::new(file_bytes))?;
let document = archive.by_name("word/document.xml")?;
let mut xml = String::new();
document.read_to_string(&mut xml)?;  // ← aucune limite
```

**Problème** : `read_to_string` charge l'intégralité du `word/document.xml` décompressé en RAM. Un DOCX malveillant de quelques Ko peut se décompresser en plusieurs Go → **OOM / DoS**.

**Recommandation** :
- Lire en streaming avec une limite stricte (ex. `MAX_DOCX_XML_BYTES = 10 MiB`).
- Vérifier le ratio de compression (`uncompressed_size / compressed_size`) et rejeter si > 100.
- Idéalement, compter les caractères extraits et couper à une limite (ex. 1 M chars).

---

### 2.2 Aucune protection zip bomb — XLSX en lecture/écriture

**Fichiers** :
- `tool_spreadsheet_calamine.rs:41` (`open_workbook_auto`)
- `tool_spreadsheet_write_edit.rs:9` (`umya_spreadsheet::reader::xlsx::read`)

**Problème** : calamine et umya_spreadsheet chargent le xlsx sans garde sur la taille décompressée. Un XLSX malveillant (écrit par l'agent puis ré-édité) peut exploser en mémoire.

**Recommandation** :
- Vérifier la taille du fichier source avant ouverture (`metadata().len() < MAX_XLSX_SOURCE_BYTES`, ex. 50 MiB).
- Pour calamine, inspecter les `metadata` du zip avant parse (taille décompressée déclarée).
- Limiter le nombre de cellules lus via les bornes existantes (`HARD_MAX_ROWS`/`HARD_MAX_COLS`).

---

### 2.3 Image process — dimension resize non bornée

**Fichier** : `tool_image_process.rs:92-107`

```rust
let w = args["w"].as_u64().unwrap_or(0) as u32;
let h = args["h"].as_u64().unwrap_or(0) as u32;
// ... resize_exact(w, h, ...)
```

**Problèmes** :
1. `w`/`h` peuvent atteindre `u32::MAX` → `resize_exact(u32::MAX, …)` tente une allocation massive → **OOM/panic**.
2. `image::open` (l.35) décode l'image entière en RAM sans limite de dimensions : un PNG 50000×50000 (~10 Go RGBA) → **OOM**.

**Recommandation** :
- Clamper `w`/`h` à une valeur raisonnable (ex. `MAX_DIMENSION = 8000`).
- Après `image::open`, vérifier `image.dimensions()` et rejeter si `w * h > MAX_PIXELS` (ex. 50 M pixels).
- Idéalement, lire les dimensions via `image::io::Reader::new(...).with_guessed_format()?.into_dimensions()` avant décodage complet, et rejeter tôt.

---

### 2.4 CSV lu entièrement en RAM avant la limite `max_rows`

**Fichier** : `tool_spreadsheet_read.rs:90`

```rust
let content = std::fs::read_to_string(resolved)?;  // ← charge tout
```

**Problème** : la limite `max_rows=5000` ne s'applique qu'au **résultat**, pas à la lecture. Un CSV de plusieurs Go → **OOM avant la boucle de limitation**.

**Recommandation** :
- Passer en streaming via `csv::ReaderBuilder` + `BufReader::new(File)`.
- Compter les lignes lues et stopper dès que `max_rows` est atteint.
- Ajouter aussi une borne sur la taille totale lue (ex. 50 MiB).

---

### 2.5 Blocklist de commandes destructives contournable

**Fichier** : `security.rs:5-23` (`check_destructive_command`)

**Problème** : match par sous-chaîne exacte. Contournements triviaux :
- `rm -rf /` bloqué, mais `rm  -rf  /` (espaces multiples) passe.
- `find / -delete`, `rsync -a --delete / /tmp/x`, `dd if=/dev/zero of=/dev/sda`, `mkfs.ext4 /dev/sda` ne sont pas couverts.
- Variables : `rm -rf $HOME`, `rm -rf ~` non couverts.

**Recommandation** :
- Compléter la blocklist avec les patterns manquants (au moins `find .* -delete`, `rsync .* --delete`, `dd .* of=/dev/`, `mkfs`, `> /dev/sd`).
- Reconnaître explicitement qu'une blocklist ne sera jamais exhaustive et que la **vraie défense** reste le permission gate en mode manuel (déjà en place).
- Documenter cette limite dans le code.

---

## 3. Problèmes haute sévérité

### 3.1 TOCTOU sur `write_file`

**Fichier** : `tool_files.rs:72-85`

```rust
create_dir_all(parent)?;          // ← création avant validation finale
// ... re-vérification du parent canonicalisé ...
if path.is_symlink() { return err }
tokio::fs::write(path, content)   // ← TOCTOU : symlink substituable entre les deux
```

**Problèmes** :
1. `create_dir_all` exécuté **avant** la validation finale → des répertoires peuvent être créés puis l'écriture refusée (effet de bord persistant).
2. Check `is_symlink()` puis `write` sans `O_NOFOLLOW` → un symlink peut être substitué entre les deux pour écrire hors zone.

**Recommandation** :
- Déplacer `create_dir_all` **après** toutes les validations.
- Utiliser `tokio::fs::OpenOptions::new().write(true).create_new(true).open(path)` avec `.custom_flags(O_NOFOLLOW)` (Unix) pour ouvrir sans suivre les liens.
- Ou : ré-évaluer `is_symlink()` juste avant l'écriture et utiliser `tokio::fs::symlink_metadata`.

---

### 3.2 ReDoS sur `grep`

**Fichier** : `tool_grep.rs:55` + `tool_scan_timeout.rs:31-40`

**Problème** :
- `RegexMatcher::new(pattern)` accepte n'importe quelle regex. Un pattern catastrophique (backtracking exponentiel) peut bloquer le thread.
- `tool_scan_timeout.rs` ne fait qu'un `AtomicBool::store(true)` au timeout, **sans `JoinHandle::abort`** → le thread `spawn_blocking` n'est pas interrompu, il continue jusqu'à consulter le flag.

**Recommandation** :
- Utiliser un moteur regex sans backtracking (ex. `regex` crate qui est déjà RE2-based — vérifier que `grep_matcher` n'utilise pas `fancy-regex`).
- Si backtracking inévitable : garder le `JoinHandle`, appeler `.abort()` au timeout pour libérer le thread.
- Ajouter une limite sur la complexité du pattern ou un timeout CPU.

---

### 3.3 Timeout agrégé absent sur batch parallèle

**Fichier** : `tool_executor_parallel_batch.rs:63`

```rust
let results = futures::future::join_all(tasks).await;  // ← pas de timeout global
```

**Problème** : si un read (web_fetch lent, read_file sur NFS pendant, grep ReDoS) bloque, tout le chunk de 10 reste bloqué. Le `cancel.is_cancelled()` (`tool_executor_parallel.rs:30`) n'est vérifié qu'**entre** chunks, jamais pendant l'attente.

**Recommandation** :
- Encapsuler dans `tokio::time::timeout(Duration::from_secs(N), join_all(tasks))`.
- Ou ajouter un `tokio::select!` sur `cancel.cancelled()` pendant le `join_all`.
- Propager un `ToolResult::err("timeout")` pour les tasks non terminées.

---

### 3.4 MCP call sans timeout

**Fichier** : `tool_mcp.rs:138`

```rust
connector.transport.call_tool(...).await  // ← pas de timeout
```

**Problème** : un connecteur MCP planté peut bloquer la session indéfiniment.

**Recommandation** :
- `tokio::time::timeout(Duration::from_secs(60), connector.transport.call_tool(...))`.
- Au timeout, retourner `ToolResult::err("Le connecteur MCP n'a pas répondu dans les temps.")`.

---

### 3.5 Threads `spawn_blocking` qui fuient après timeout

**Fichier** : `tool_scan_timeout.rs:31-40`

**Problème** : aucun `JoinHandle::abort()` → accumulation possible de threads zombies en cas de scans lents répétés (notamment grep ReDoS ou lecture NFS pendante).

**Recommandation** :
- Conserver le `JoinHandle` retourné par `spawn_blocking`.
- Au timeout, appeler `handle.abort()` pour libérer le thread (le travail en cours sera interrompu à la prochaine await point).

---

## 4. Problèmes moyenne sévérité

### 4.1 `grep`/`read_file`/`glob`/`list_dir` non redactés en post-hook

**Fichier** : `tool_hooks.rs:64-71`

```rust
pub fn run_post_hooks(tool_name: &str, _args: &Value, result: ToolResult) -> ToolResult {
    if tool_name == "bash" {
        return ToolResult { content: redact_text(&result.content), ..result };
    }
    result  // ← read_file, grep, glob, list_dir ne sont JAMAIS redactés
}
```

**Problème** : un `read_file ~/.env` ou un `grep` matchant `.aws/credentials` retourne le contenu **en clair** au LLM. Combiné à `WalkBuilder::hidden(false)` dans glob/grep, les fichiers cachés sont exposés.

**Recommandation** :
- Étendre `run_post_hooks` pour appliquer `redact_text` sur `read_file`, `grep`, `glob`, `list_dir`.
- Renforcer `redact_text` (`sensitive_data.rs`) : ajouter un pattern pour les blocs PEM (`-----BEGIN ... PRIVATE KEY-----`), les fichiers `.pem`, certificats.

---

### 4.2 Pré-hook `..` absent pour `glob`/`grep`/`list_dir`/`bash`

**Fichier** : `tool_hooks.rs:25-44`

**Problème** : seuls `write_file`/`edit_file`/`read_file`/`write_spreadsheet`/`write_document`/`process_image` sont vérifiés contre `..`. Pour les autres, seule `validate_read_path` (canonicalisation) protège — défense en profondeur manquante.

**Recommandation** :
- Étendre le `matches!` à `list_dir`, `glob`, `grep` (vérifier le champ `path`).
- Pour `bash`, vérifier la présence de `..` dans la commande (heuristique, faux positifs possibles — à évaluer).

---

### 4.3 `interactive_choice_gate` global cross-session

**Fichier** : `interactive_choice_gate.rs:17-18, 51`

**Problème** :
- `PENDING` est une `static` partagée entre toutes les sessions.
- `respond(id, answers)` ne vérifie pas l'appartenance de l'id à la session appelante.
- Quota `MAX_PENDING=64` global → une session buggy peut épuiser le quota et bloquer toutes les UI interactives (DoS soft).

**Recommandation** :
- Clé composite `(session_id, id)` au lieu de `id` seul.
- Vérifier en début de `respond` que la session correspondante est bien l'appelante.
- Quota par session (ex. 4 pending max/session) en plus du global.

---

### 4.4 `tool_plan_guard::ensure_allowed_for_session` sans lock

**Fichier** : `tool_plan_guard.rs:43-54`

**Problème** : lit `plan_mode_enabled` sans verrou, alors que `write_plan` prend le lock (`tool_plan.rs:82-86`). Race TOCTOU : un tool d'écriture pourrait s'exécuter si le mode plan s'active entre le check et l'exécution.

**Recommandation** :
- Effectuer le check sous le même `session_store::lock_session` que l'exécution.
- Ou : vérifier `plan_mode_enabled` à nouveau juste avant le dispatch effectif.

---

### 4.5 WriteGuard contournable pour office/image

**Fichier** : `tool_executor_helpers.rs:13`

**Problème** : `check_write_guard` ne s'applique qu'à `write_file`/`edit_file`. `write_spreadsheet`, `write_document`, `process_image` ne sont jamais vérifiés contre les chemins lus — la protection "read-before-write" est incohérente.

**Recommandation** :
- Étendre `check_write_guard` à `write_spreadsheet`, `write_document`, `process_image`.

---

### 4.6 XML malformé DOCX → texte tronqué silencieux

**Fichier** : `tool_document_read.rs:122`

```rust
Ok(Event::Err(_)) => break,  // ← arrêt silencieux, retourne le texte partiel
```

**Problème** : sur XML malformé, le parser s'arrête sans erreur et renvoie le texte déjà accumulé → **donnée corrompue non détectée**.

**Recommandation** :
- Au premier `Event::Err`, retourner une erreur explicite ou au moins ajouter un marqueur `[document potentiellement corrompu]` au texte retourné.

---

### 4.7 Formula injection XLSX

**Fichiers** : `tool_spreadsheet_write_edit.rs:118-119`, `tool_spreadsheet_write_new.rs:105-107`

**Problème** : aucune validation/sanitization des formules écrites. L'agent peut écrire `=HYPERLINK("http://attaquant")`, des références externes ou des appels DDE qui s'exécutent à l'ouverture dans Excel.

**Recommandation** :
- Préfixer les valeurs commençant par `=`, `+`, `-`, `@` par une apostrophe `'` (convention Excel pour forcer le texte), sauf si la valeur est explicitement passée comme formule par l'agent.
- Ou : émettre un avertissement dans le `ToolResult` si une formule est détectée.

---

### 4.8 Log office non rotatif

**Fichier** : `tool_dispatcher_office.rs:5-23`

**Problème** : `tool-calls.jsonl` en append-only sans rotation ni taille max → croissance disque non bornée.

**Recommandation** :
- Rotation par taille (ex. 10 MiB) comme pour `logs/wakeups.jsonl` (rolling 500 lignes).
- Ou : ne loguer que les métadonnées (tool name, timestamp) sans les args complets.

---

### 4.9 Budget de tokens contournable via fichiers persistés

**Fichiers** : `tool_dispatcher.rs:51, 58-61` + `tool_result_budget.rs`

**Problème** : quand un résultat dépasse le seuil, le full output est écrit sur disque (`persist_result`) et le chemin **absolu** est inclus dans la preview retournée au LLM. Au tour suivant, le LLM peut `read_file` ce chemin pour récupérer l'intégralité, contournant `MAX_TOTAL_RESULT_CHARS`.

**Recommandation** :
- Conserver le comportement (c'est utile) mais être conscient que le budget est une **soft-limit**.
- Option : stocker les gros résultats hors d'une racine lisible par `read_file`, ou exiger une confirmation pour les relire.

---

### 4.10 `execute_background_shell` est `pub` et n'a aucune validation destructive

**Fichier** : `tool_bash_long.rs:36`

**Problème** : la fonction est `pub` et n'appelle pas `check_destructive_command`. Ce n'est sûr que parce qu'elle est appelée depuis `tool_bash::execute_shell` **après** le check. Tout appel direct depuis un autre module contournerait la vérification.

**Recommandation** :
- Passer la fonction en `pub(crate)` ou `pub(super)`.
- Ajouter un commentaire `/// NE PAS appeler directement — validation destructive assurée par l'appelant`.
- Idéalement, déplacer le check destructive à l'intérieur de `execute_background_shell` lui-même.

---

## 5. Problèmes basse sévérité

### 5.1 Information disclosure : chemin absolu filesystem

**Fichier** : `tool_dispatcher.rs:58-61`

Le chemin absolu complet (`data_dir()/tool-results/...`) est renvoyé au LLM dans la preview des gros résultats, exposant la structure de chemins locale.

**Recommandation** : afficher un chemin relatif ou masqué plutôt qu'absolu.

---

### 5.2 `repair_json` heuristique hasardeuse

**Fichier** : `tool_spreadsheet_write.rs:51-63`

Transformations (`replace('\'', "\"")`, ajout de `]`/`}`) pouvant interpréter un JSON invalide différemment de l'intention → données corrompues silencieuses.

**Recommandation** : soit échouer fermé sur JSON invalide, soit logger un avertissement quand une réparation est appliquée.

---

### 5.3 `redact_text` regex incomplète

**Fichier** : `sensitive_data.rs`

Pas de pattern pour :
- Les blocs PEM (`-----BEGIN ... PRIVATE KEY-----`).
- Les fichiers `.pem`, certificats.
- Les clés privées SSH brutes.

**Recommandation** : compléter les patterns de redaction.

---

### 5.4 `is_ip_private` moins strict que `ssrf::is_blocked_ip`

**Fichier** : `tool_web_fetch_ip.rs:18-38`

Omet CGNAT (100.64/10), multicast, 0.0.0.0/8, 240/4. Utilisé par `link_preview/security.rs`, pas par l'agent.

**Recommandation** : harmoniser en réutilisant `ssrf::is_blocked_ip`, ou supprimer ce helper redondant.

---

### 5.5 `ssrf::is_blocked_host_literal` trop large

**Fichier** : `ssrf.rs:112-126`

`host.starts_with("fd")` génère des faux positifs sur des domaines légitimes (`fdns…`). Pas de bypass (le DNS resolve + `is_blocked_ip` couvre), nuisance uniquement.

**Recommandation** : affiner la règle ou la supprimer si redondante.

---

### 5.6 `normalize_formula` traduit naïvement dans les chaînes littérales

**Fichier** : `tool_office_utils.rs:88-99`

`"SOMME("` dans un texte cellulaire devient `"SUM("` → corruption silencieuse de contenu.

**Recommandation** : parser pour ne remplacer que hors des chaînes littérales, ou abandonner la traduction FR→EN.

---

### 5.7 WebP quality ignorée silencieusement

**Fichier** : `tool_image_process.rs:143-151`

L'option `quality` n'a aucun effet sur WebP (image 0.25 ne supporte que lossless), mais aucune indication n'est renvoyée.

**Recommandation** : inclure un avertissement dans le `ToolResult` quand la quality est demandée sur un WebP.

---

### 5.8 Troncation silencieuse u64→u32→u16

**Fichier** : `tool_office_utils.rs:20-33`

`as u32` / `as u16` sans garde → row/col absurdes sans erreur sur très grandes valeurs.

**Recommandation** : utiliser `u32::try_from().map_err(...)` et échouer fermé.

---

### 5.9 Erreurs silencieuses multiples

- `list_dir` (`tool_files.rs:143`) : `continue` sur erreur d'entry.
- `glob`/`grep` (`tool_grep.rs:96`) : `continue` sur erreur d'entry.
- `edit_file` (`tool_files.rs:112`) : `unwrap_or(0)` mort/incohérent.
- `_ => {}` pour types de blocs/opérations inconnus dans `tool_document_write_xml.rs:37`, `tool_spreadsheet_write_edit.rs:24`, `tool_spreadsheet_write_new.rs:44`.

**Recommandation** : au minimum logger les erreurs ignorées ; idéalement retourner un avertissement agrégé dans le `ToolResult`.

---

### 5.10 `extract_persist_path` parsing fragile

**Fichier** : `tool_result_budget.rs:29-33`

Recherche du premier `]` ; si un path contient `]`, parsing incorrect (lien perdu dans le placeholder). Bénin mais bug réel.

**Recommandation** : utiliser un délimiteur plus robuste ou stocker le path séparément.

---

### 5.11 Skill loader : nom parsé non revalidé

**Fichier** : `tool_skill_loader.rs:62-83`

`find_skill_dir_by_name` compare aussi le nom parsé depuis le contenu du fichier de skill ; un skill malveillant local pourrait avoir un `name:` front-matter contenant `..` (le check l.46 ne s'applique qu'à l'arg entrant).

**Recommandation** : réappliquer la validation `contains("..")` sur le nom parsé.

---

### 5.12 `bash` cwd update détaché et silencieux

**Fichier** : `tool_dispatcher.rs:96-106`

`tokio::spawn` fire-and-forget ; erreur seulement `eprintln!`. Si l'update échoue, la session conserve un `working_dir` obsolète sans signal au LLM ni à l'UI.

**Recommandation** : soit await l'update inline, soit propager l'erreur dans le `ToolResult`.

---

## 6. Plan de correction recommandé

### Phase 1 — Critique (sécurité / DoS)

1. **Zip bomb DOCX** (§2.1) : streaming + limite taille XML.
2. **Zip bomb XLSX** (§2.2) : vérif taille source + métadonnées zip.
3. **Image resize non bornée** (§2.3) : clamp dimensions + check avant décodage.
4. **CSV en streaming** (§2.4) : ne plus `read_to_string`.
5. **Blocklist destructive** (§2.5) : compléter les patterns + documenter la limite.

### Phase 2 — Haute (robustesse / timeouts)

6. **TOCTOU write_file** (§3.1) : `O_NOFOLLOW` + reorder `create_dir_all`.
7. **ReDoS grep** (§3.2) : `JoinHandle::abort()` au timeout + vérifier le moteur regex.
8. **Timeout agrégé batch** (§3.3) : `tokio::time::timeout` sur `join_all`.
9. **Timeout MCP call** (§3.4).
10. **Abort threads spawn_blocking** (§3.5).

### Phase 3 — Moyenne (durcissement)

11. **Redaction étendue** (§4.1) : post-hook sur read/grep/glob/list_dir + patterns PEM.
12. **Pré-hook `..` étendu** (§4.2).
13. **interactive_choice_gate isolation** (§4.3) : clé `(session_id, id)`.
14. **Plan guard sous lock** (§4.4).
15. **WriteGuard étendu** (§4.5) : office/image.
16. **XML Err → erreur explicite** (§4.6).
17. **Formula injection XLSX** (§4.7).
18. **Log office rotation** (§4.8).
19. **execute_background_shell visibilité** (§4.10).

### Phase 4 — Basse (qualité / cosmétique)

20-30. Voir §5, à traiter au fil de l'eau.

---

## Notes finales

- **Tous les tools échouent fermé** (fail-closed) dans la grande majorité des cas : validation stricte, `Result<_, String>` propagé en `ToolResult::err`, messages génériques côté todo/plan.
- **Pas de panic observée** sur entrées malformées (pas d'`unwrap` sur du contenu externe, `.get()` partout, `saturating_add`, `unwrap_or_default`).
- L'architecture est **saine** : les problèmes identifiés sont des angles morts localisés (limite tailles, timeouts, redaction), pas des défauts de conception structurels.
- Le périmètre **web_fetch / plan mode / sous-agents / todo** est **particulièrement solide** — c'est sur les **tools office/image/fichiers** que se concentrent les corrections à apporter.
