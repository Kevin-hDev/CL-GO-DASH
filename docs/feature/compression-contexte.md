# Compression de contexte

Système de compression automatique et manuelle qui résume la conversation quand le contexte approche de la limite de tokens. Inspiré de la compaction de Claude Code, adapté à l'architecture multi-provider de CL-GO-DASH.

## Vue d'ensemble

Quand le nombre de tokens utilisés dépasse un seuil configurable, le LLM résume la conversation en un message structuré. Les anciens messages sont remplacés par ce résumé, libérant de l'espace pour continuer la conversation. La dernière réponse du LLM est toujours préservée.

## Modes de compression

### Compression automatique

Se déclenche automatiquement quand `context_tokens >= seuil% × fenêtre_configurée`. Vérifié à deux moments :
- **Pré-requête** : avant chaque appel LLM dans la boucle agent
- **Post-réponse** : après chaque réponse du LLM (pour couvrir le cas où le seuil est franchi pendant la génération)

### Compression manuelle

L'utilisateur tape `/compress` dans le chat. La commande est interceptée dans `agent_chat_task.rs` avant la boucle agent. Le LLM résume la conversation et le résultat remplace les messages.

## Settings (Paramètres > Avancé)

| Setting | Type | Défaut | Description |
|---------|------|--------|-------------|
| `compression_enabled` | `bool` | `true` | Active/désactive la compression |
| `compression_threshold` | `u8` | `85` | Pourcentage (0-100) de la fenêtre configurée |

## Éligibilité des modèles

La compression est disponible uniquement pour les modèles dont la capacité **native** est >= 128k tokens (`MIN_NATIVE_CONTEXT = 131_072`). C'est la capacité réelle du modèle qui compte, pas la fenêtre configurée par l'utilisateur.

Le seuil de déclenchement est calculé sur la fenêtre **configurée** (modelfile `num_ctx` ou `OLLAMA_CONTEXT_LENGTH` hardware).

Exemple : Gemma 4 E2B (natif 131k) configuré à 32k, seuil 85% → compression à 27 648 tokens.

## Détection de la fenêtre de contexte

### Ollama (`context_resolve::resolve_ollama`)

1. Appelle `/api/show` pour récupérer `model_info`
2. `native` = `model_info["{arch}.context_length"]` (architecture dynamique via `general.architecture`)
3. `configured` = `num_ctx` du modelfile si présent, sinon `min(native, compute_default_num_ctx())` (détection hardware VRAM)

### Providers API (`context_resolve::resolve_api`)

`native = configured` = `max_input_tokens` depuis le registre LiteLLM.

## Comptage de tokens

### Pendant le streaming

Chaque chunk Ollama = 1 token. Le compteur `token_count` est incrémenté dans `ollama_stream_process.rs`. Le chunk final (`done: true`) contient `eval_count` (output) et `prompt_eval_count` (prompt).

### Anneau de contexte (frontend)

Utilise `context_tokens` du `Done` event = `prompt_eval_count + eval_count` du **dernier tour** de la boucle agent. Représente la taille réelle du contexte après la réponse.

### Décision de compression

- **Premier tour** : `estimate_tokens(messages)` = `bytes / 4` (fallback)
- **Tours suivants** : `last_prompt + last_eval` (valeurs réelles Ollama du tour précédent)

## Prompt de compression

Le prompt est en anglais, structuré en 9 sections. Envoyé comme dernier message `user` avec interdiction d'utiliser des tools.

### Structure

```
[PREAMBLE] — CRITICAL: TEXT ONLY, no tools
[ANALYSIS INSTRUCTION] — Bloc <analysis> comme brouillon
[BASE PROMPT] — 9 sections demandées :
  1. Primary Request and Intent
  2. Key Technical Concepts
  3. Files and Code Sections
  4. Errors and Fixes
  5. Problem Solving
  6. All User Messages
  7. Pending Tasks
  8. Current Work
  9. Next Step
[TRAILER] — REMINDER: no tools
```

### Post-traitement

`extract_summary()` extrait le contenu de `<summary>...</summary>`, supprime le bloc `<analysis>`. Fallback sur le texte brut si pas de tags.

### Message réinjecté

```
This session is being continued from a previous conversation that ran out of
context. The summary below covers the earlier portion of the conversation.

[résumé]

Continue the conversation from where it left off without asking the user any
further questions. Resume directly — do not acknowledge the summary, do not
recap what was happening, do not preface with "I'll continue" or similar.
```

## Animation

Lottie violet (même loader que le thinking) + texte "Compression" en orange (`#f97316`) avec pulsation CSS 2s. Positionné en bas du chat, après tout le contenu streaming.

## Flux auto-compression détaillé

```
1. Boucle agent : tour N
2. LLM génère sa réponse
3. messages.push(assistant_message)
4. last_prompt + last_eval calculés
5. try_auto_compress(last_prompt + last_eval)
   a. Vérifie : enabled? eligible? >= seuil?
   b. Si oui :
      - Émet Compressing { start }
      - Capture le dernier message assistant
      - Envoie toute la conversation au LLM pour résumé (collect_chat, sans streaming UI)
      - extract_summary() sur la réponse
      - apply_compression() : remplace les messages en mémoire
      - Rajoute le dernier message assistant
      - save_compressed_session() : sauvegarde résumé + dernière réponse dans le session store
      - Émet Compressing { done }
      - Émet CompressionComplete
6. Frontend reçoit CompressionComplete → recharge session depuis le store
7. UI mise à jour : anciens messages remplacés par résumé + dernière réponse
8. Boucle agent continue (ou Done si pas de tool calls)
```

## Flux /compress détaillé

```
1. User tape /compress → sélectionne dans l'autocomplete → texte mis dans le champ
2. User appuie Enter → chat.sendMessage("/compress")
3. Backend : agent_chat_task.rs détecte is_compress_command()
4. handle_compress_command() :
   a. Émet Compressing { start }
   b. Filtre /compress des messages
   c. Envoie au LLM pour résumé
   d. extract_summary()
   e. Sauvegarde session compressée (1 message résumé)
   f. Émet Compressing { done }
   g. Émet CompressionComplete
   h. Émet Done (context_tokens = tokens du résumé)
5. Frontend recharge session → affiche le résumé
```

## Fichiers — Backend Rust

| Fichier | Rôle |
|---------|------|
| `services/compress/mod.rs` | Module principal, exports |
| `services/compress/engine.rs` | `should_auto_compress`, `apply_compression`, `build_post_compression_messages` |
| `services/compress/engine_tests.rs` | 13 tests unitaires engine |
| `services/compress/prompt.rs` | Prompt 9 sections, `extract_summary`, `format_summary_message` |
| `services/compress/token_estimate.rs` | `estimate_tokens` (bytes/4), `should_compress` |
| `services/compress/eligibility.rs` | `is_model_eligible` (>= 128k natif) |
| `services/compress/context_resolve.rs` | `resolve_ollama`, `resolve_api` — détecte native/configured |
| `services/compress/types.rs` | `CompressionConfig`, `CompressionResult` |
| `services/compress/integration_tests.rs` | 10 tests d'intégration |
| `services/agent_local/compress_hook.rs` | Hook auto-compression Ollama + sauvegarde session |
| `services/llm/compress_hook.rs` | Hook auto-compression LLM API + sauvegarde session |
| `services/llm/stream_silent.rs` | Stream silencieux (pas d'events UI) pour compression API |
| `services/llm/retry.rs` | Logique retry extraite du agent loop |
| `commands/agent_chat_task.rs` | Interception `/compress`, `handle_compress_command` |
| `models/config.rs` | `compression_enabled`, `compression_threshold` dans `AdvancedSettings` |
| `services/agent_local/types_ollama.rs` | `StreamEvent::Compressing`, `StreamEvent::CompressionComplete`, `context_tokens` dans `Done` |

## Fichiers — Frontend TypeScript

| Fichier | Rôle |
|---------|------|
| `hooks/use-compression.ts` | Hook écoute `Compressing` events, expose `isCompressing` |
| `hooks/use-slash-commands.ts` | Built-in commands (`/compress`), fusion avec skills |
| `hooks/use-active-skills.ts` | Gestion built-in : met `/compress` dans le champ |
| `hooks/agent-chat-stream-callbacks.ts` | `finalizeStream` utilise `contextTokens` pour l'anneau |
| `hooks/agent-stream-manager.ts` | Reload session sur `CompressionComplete` |
| `components/agent-local/compression-indicator.tsx` | Lottie + "Compression" orange |
| `components/agent-local/compression-indicator.css` | Pulsation CSS |
| `components/agent-local/message-list.tsx` | Affichage conditionnel `CompressionIndicator` en bas |
| `components/settings/advanced-settings.tsx` | Toggle + slider compression |
| `types/agent.ts` | `contextTokens` dans le type `StreamEvent` Done |

## Tests

235 tests au total dont :
- 5 tests éligibilité modèle
- 6 tests estimation tokens
- 11 tests prompt
- 13 tests engine
- 10 tests intégration
- 2 tests config
