# Providers LLM API — Bugs résolus et apprentissages

Session du 20 avril 2026. Corrections majeures sur la parité Ollama/API,
la détection des capabilities, la gestion des images, et la propagation
des erreurs.

---

## 1. System prompt — Les LLM se comportaient en chatbot

### Symptôme
Les modèles locaux (Gemma 4, Qwen 3.5) refusaient d'utiliser bash et
de sortir du répertoire de travail. Ils répondaient "je n'ai pas accès
à votre système" au lieu d'utiliser leurs outils.

### Cause racine
Le system prompt disait "You are working in: X / All file paths are
relative to this directory" — les petits modèles interprétaient ça
comme une prison. Le ton ultra-restrictif ("Use a tool ONLY when...",
"is it truly necessary?") renforçait le refus.

### Ce qui a fonctionné
- Réécriture complète du prompt : "You are an autonomous coding agent
  with full access to the user's system"
- Working directory présenté comme "default starting point, not a boundary"
- Deux niveaux de prompt selon la taille du modèle :
  - Compact (<25B) : ~350 mots, direct
  - Detailed (≥25B) : ~850 mots, sections git/code/error handling
- Détection taille par parsing du nom du modèle (`model_size.rs`)

### Ce qui n'a PAS fonctionné
- Le prompt en français : les LLM suivent mieux les instructions en anglais
  pour le tool calling (tous entraînés sur des données anglaises)
- Garder "ONLY when" même avec d'autres changements : le ton restrictif
  suffit à bloquer les petits modèles

### Fichiers
- `src-tauri/src/services/agent_local/prompt_compact.rs`
- `src-tauri/src/services/agent_local/prompt_detailed.rs`
- `src-tauri/src/services/agent_local/model_size.rs`
- `src-tauri/src/services/agent_local/chat_prompts.rs`

---

## 2. Thinking/Réflexion mort côté API

### Symptôme
Le paramètre `think: true` envoyé par le frontend était silencieusement
ignoré pour tous les providers API. Aucun `StreamEvent::Thinking` émis.
Les modèles reasoning (DeepSeek-R1, Qwen-QwQ) ne montraient pas leur
raisonnement.

### Cause racine
La pipeline API (`llm/agent_loop.rs → stream.rs → stream_http.rs`)
n'acceptait pas le paramètre `think`. Il s'arrêtait à `agent_chat.rs`.

### Ce qui a fonctionné
- Propagation de `think` dans toute la pipeline API
- Extraction de `delta.reasoning_content` (DeepSeek/Groq) dans les chunks SSE
- Émission de `StreamEvent::Thinking` côté API comme Ollama
- Nettoyage des tags `<think>` dans le contenu (Qwen via OpenRouter)

### Apprentissage clé — format thinking par provider
| Provider | Champ payload | Champ SSE response |
|---|---|---|
| DeepSeek | `reasoning_effort: "high"` | `delta.reasoning_content` |
| OpenAI (o3/o4) | `reasoning: {effort: "high"}` | `delta.reasoning` |
| Google (OAI-compat) | `reasoning_effort: "high"` | Non documenté |
| Ollama | `think: true` dans la requête | `msg.thinking` dans le chunk |

### ERREUR COMMISE — Google `generationConfig`
On avait mis `generationConfig: {thinkingConfig: {thinkingBudget: 0}}`
pour Google. C'est le champ de l'API NATIVE Google, pas de l'endpoint
OpenAI-compatible. L'endpoint OAI-compat de Google attend
`reasoning_effort`. Ça cassait TOUTES les requêtes Google quand
thinking était activé.

### Fichiers
- `src-tauri/src/services/llm/stream_http.rs` (payload thinking)
- `src-tauri/src/services/llm/stream.rs` (parsing `reasoning_content`)
- `src-tauri/src/services/llm/tool_capable.rs` (`supports_thinking()`)

---

## 3. Images/Vision — Multiples bugs cumulés

### Symptôme
- Certains modèles avec badge "V" ne voyaient pas les images
- Certains modèles bloquaient silencieusement (erreur 400/404)
- Ministral retournait 429 rate limit sur les images
- OpenRouter 404 "No endpoints found that support image input"

### Causes racines (multiples)

**3a. Format image Mistral différent**
Mistral attend `image_url` comme **string directe** :
```json
{"type": "image_url", "image_url": "data:image/png;base64,..."}
```
Tous les autres (OpenAI, Groq, Google, OpenRouter) attendent un **objet** :
```json
{"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}}
```
On envoyait le format objet à tout le monde → Mistral rejetait.

**3b. Images envoyées aux messages system**
Le code convertissait TOUS les messages avec images (y compris system)
en content array. Certains providers rejettent le content array sur les
messages non-user avec "content must be a string".

**3c. Badge "V" faux — OpenRouter**
Le parser lisait `architecture.modality` et matchait `contains("image")`.
Ça matchait `"text->image"` (génération d'images, DALL-E style) en plus
de `"image->text"` (vision). Résultat : des modèles de génération
d'images marqués comme ayant la vision.

**3d. Badge "V" faux — hardcoded**
`tool_capable.rs` avait :
- `"openrouter" => true` pour vision (TOUS les modèles)
- `"deepseek" => model.contains("chat")` (faux, deepseek-chat n'a pas vision)

**3e. LiteLLM ne corrigeait pas les faux positifs**
La logique `if !m.supports_vision { m.supports_vision = caps.supports_vision }`
n'upgradeait que false→true, jamais true→false. Quand l'API parser
disait "vision=true" à tort, LiteLLM ne pouvait pas corriger.

### Ce qui a fonctionné
- Format image adapté par provider dans `stream_convert.rs`
- Images converties uniquement pour les messages `user`
- Parser OpenRouter : `s.contains("image->") || s.contains("image+")`
- Hardcoded OpenRouter/DeepSeek corrigé
- LiteLLM override le parser API : `m.supports_vision = caps.supports_vision`
  (confiance au registre pour corriger les faux positifs)
- Strip des images pour modèles sans vision + note texte

### Fichiers
- `src-tauri/src/services/llm/stream_convert.rs`
- `src-tauri/src/services/llm/openai_compat_parsing.rs`
- `src-tauri/src/services/llm/tool_capable.rs`
- `src-tauri/src/commands/agent_chat.rs`

---

## 4. Capabilities detection — Model IDs préfixés

### Symptôme
Llama 4 Scout et Qwen3-32B sur Groq n'avaient pas le badge "T"
(tools) alors qu'ils les supportent.

### Cause racine
Les model IDs Groq utilisent le format `meta-llama/llama-4-scout-17b`
(avec préfixe org). Nos patterns matchaient `model.starts_with("llama-4")`
mais l'ID commençait par `meta-llama/`.

### Fix
`strip_org_prefix()` appliqué dans les 3 fonctions de détection.
Extrait la partie après le dernier `/`.

---

## 5. OpenRouter — Routing multi-backend

### Symptôme
Des modèles avec le badge "T" retournaient 404 "No endpoints found
that support tool use" de manière intermittente (3 sur 5 messages OK,
2 en erreur sur le même modèle).

### Cause racine
OpenRouter route chaque requête vers un backend différent (Google AI
Studio, Together, Fireworks...). Un modèle peut lister "tools" dans
ses metadata (`supported_parameters`) mais c'est une **capability
agrégée** — au runtime, le backend sélectionné peut ne pas la supporter.

Cas aggravants :
- Modèles `:free` — les endpoints gratuits supportent rarement les tools
- `require_parameters: true` peut sur-filtrer les backends

### Ce qui a fonctionné
- `require_parameters: true` + `allow_fallbacks: true` dans le payload
- Retry automatique sans tools sur 404 "tool use"
- Retry automatique sans images sur 400/404 "image"
- Classification des erreurs : `Fatal` vs `RetryWithoutTools` vs
  `RetryWithoutImages`

### Apprentissage — erreurs spécifiques OpenRouter
| Erreur | Cause | Action |
|---|---|---|
| 404 "No endpoints found that support tool use" | Aucun backend dispo avec tools | Retry sans tools |
| 404 "No endpoints found that support image input" | Aucun backend dispo avec vision | Retry sans images |
| 400 "Developer instruction is not enabled" | Google AI Studio rejette le system prompt | Erreur fatale, changer de modèle |
| 400 "content must be a string" | Format content array non supporté | Retry sans images |
| 400 "Request contains an invalid argument" | Paramètre non supporté par le backend | Erreur fatale |

### Fichiers
- `src-tauri/src/services/llm/stream_http.rs` (`RequestError` enum)
- `src-tauri/src/services/llm/stream.rs` (retry logic)

---

## 6. Erreurs silencieuses — Jamais affichées dans l'UI

### Symptôme
Quand une requête API échouait (400, 404, 429...), l'UI ne montrait
rien. Le chat restait bloqué sans feedback. Les erreurs n'étaient
visibles que dans le terminal (stderr).

### Cause racine (DEUX bugs cumulés)

**6a. Message d'erreur remplacé par un générique**
Dans `agent-chat-stream-callbacks.ts`, le case `"error"` remplaçait
le message du backend par "Le flux s'est interrompu." → perte de
l'information utile.

**6b. Erreur explicitement jetée dans le hook React**
Dans `use-agent-chat.ts`, le snapshot destructurait `error` et le
jetait avec `void error` :
```typescript
const { pendingPermissions, completed, error, ...chatState } = snapshot;
void error;  // ← JETÉ
setState(chatState);  // ← chatState n'a plus error
```
L'erreur existait dans le snapshot mais était **délibérément ignorée**
avant d'atteindre le state React.

### Fix
- `event.data.message || "Le flux s'est interrompu."` (garde le message réel)
- Suppression du `void error` et du destructuring de `error`
- `error` ajouté à `ChatState` et propagé jusqu'à `MessageList`
- Affichage en rouge dans le chat

### Fichiers
- `src/hooks/agent-chat-stream-callbacks.ts`
- `src/hooks/use-agent-chat.ts`
- `src/components/agent-local/message-list.tsx`
- `src/components/agent-local/chat-view.tsx`

---

## 7. Modèles non-chat dans la liste

### Symptôme
Des modèles audio (Lyria 3, Whisper), d'embedding, de génération
d'images (DALL-E, Imagen) apparaissaient dans la liste de chat.
Certains "répondaient" (Lyria renvoyait des paroles de musique)
mais crashaient sur les tools.

### Cause racine
L'endpoint `/models` de chaque provider retourne TOUS les modèles
(chat, embedding, audio, image generation...). On ne filtrait pas.

### Ce qui a fonctionné
- Registre LiteLLM : champ `mode` (`chat`, `embedding`,
  `image_generation`, `audio_transcription`, etc.)
- Filtre : garder uniquement `mode == "chat"` ou `mode == "completion"`
- Mistral : filtre par `capabilities.completion_chat == false`
- Fallback heuristique pour modèles hors registre : noms contenant
  whisper, dall-e, tts, embedding, lyria, imagen, etc.

### Résultat du filtrage
| Provider | Avant | Filtrés | Après |
|---|---|---|---|
| Groq | 14 | 3 (whisper, playai-tts) | 11 |
| Gemini | 65 | 24 (embedding, image gen, audio) | 41 |
| Mistral | 51 | 5 (OCR, embedding) | 46 |

### Fichiers
- `src-tauri/src/services/llm/model_registry.rs` (`is_chat_model()`)
- `src-tauri/src/commands/llm.rs` (filtre dans `list_llm_models`)
- `src-tauri/src/services/llm/openai_compat_parsing.rs` (filtre Mistral)

---

## 8. Registre LiteLLM — Source de vérité pour les capabilities

### Pourquoi
Chaque provider expose les capabilities différemment :
- **Mistral** : `capabilities.function_calling`, `capabilities.vision`
- **OpenRouter** : `supported_parameters[]`, `architecture.modality`
- **Groq/Google/OpenAI/DeepSeek** : rien dans `/models`

Le hardcoding dans `tool_capable.rs` était fragile (devstral manquant,
deepseek-chat marqué vision à tort, etc.) et devenait obsolète à
chaque nouveau modèle.

### Solution
Fichier JSON public de LiteLLM (~1.3 Mo, 2672 modèles) :
`https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json`

Champs utilisés :
- `supports_vision` — booléen
- `supports_function_calling` — booléen (= tools)
- `supports_reasoning` — booléen (= thinking)
- `max_input_tokens` — context length
- `mode` — type de modèle (chat, embedding, etc.)
- `litellm_provider` — provider original

### Architecture
1. JSON embarqué dans `src-tauri/resources/litellm-models.json` (fallback offline)
2. Au démarrage, check GitHub avec `If-Modified-Since` (pas de download si pas changé)
3. Si mis à jour, cache dans `~/.local/share/cl-go-dash/litellm-models.json`
4. Lookup par `{provider}/{model_id}` avec fallback strip org prefix

### Chaîne de priorité pour les capabilities
1. Données API provider (Mistral capabilities, OpenRouter supported_parameters)
2. Registre LiteLLM (override les faux positifs du parser API pour vision)
3. Hardcoded `tool_capable.rs` (dernier recours, modèles hors registre)

### Fichiers
- `src-tauri/resources/litellm-models.json`
- `src-tauri/src/services/llm/model_registry.rs`
- `src-tauri/src/commands/llm.rs`
- `src-tauri/src/lib.rs` (init au démarrage)

---

## 9. Timeout bash et streaming stats

### Bash timeout
- Avant : 30s par défaut → les commandes `du` sur de gros répertoires timeout
- Après : 120s par défaut, max 600s (10 min), comme Claude Code
- Le LLM peut passer un timeout custom via le paramètre `timeout` de l'outil

### Streaming stats temps réel
- Timer dynamique à côté du curseur pendant la génération (reset entre les tours)
- Token count en live (thinking + tokens comptés)
- Temps total affiché à la fin : `12s · 213 tokens · 12.9 t/s`

### Tool bubble — erreurs visibles
- Message d'erreur affiché en rouge sous la commande échouée
- Avant : juste une croix ✗ rouge sans explication

---

## Résumé des apprentissages

### Formats API par provider (avril 2026)

| Aspect | Ollama | OpenAI | Groq | Mistral | Google (OAI-compat) | OpenRouter | DeepSeek |
|---|---|---|---|---|---|---|---|
| Image format | base64 dans `images[]` | `{url: "data:..."}` objet | `{url: "data:..."}` objet | `"data:..."` **string directe** | `{url: "data:..."}` objet | `{url: "data:..."}` objet | `{url: "data:..."}` objet |
| Thinking activation | `think: true` | `reasoning: {effort}` | N/A | N/A | `reasoning_effort` | Dépend du modèle | `reasoning_effort` |
| Thinking response | `msg.thinking` | `delta.reasoning` | N/A | N/A | ? | Dépend du modèle | `delta.reasoning_content` |
| Tool call format | JSON complet en 1 chunk | Fragments SSE accumulés | Complet en 1 chunk | Fragments SSE | Fragments SSE | Dépend du backend | Fragments SSE |
| Capabilities API | `/api/show` dynamique | Rien dans `/models` | Rien dans `/models` | `capabilities{}` | Rien dans `/models` | `supported_parameters[]` | Rien dans `/models` |
| Stall timeout | 5 min (ajouté) | N/A | N/A | N/A | N/A | N/A | N/A |

### Règles retenues
- Ne jamais hardcoder `true` par défaut pour des capabilities (surtout OpenRouter/Google)
- Toujours strip le préfixe org avant de matcher les patterns (`meta-llama/llama-4` → `llama-4`)
- Le registre LiteLLM est la source de vérité pour vision — il corrige les faux positifs du parser API
- Les erreurs HTTP doivent TOUJOURS remonter au frontend — jamais de `void error`
- Les retry automatiques sont essentiels pour OpenRouter (routing multi-backend)
- Les modèles non-chat doivent être filtrés de la liste (embedding, audio, image gen)
