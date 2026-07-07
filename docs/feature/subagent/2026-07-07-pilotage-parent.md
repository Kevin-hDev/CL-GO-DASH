# Pilotage des sous-agents par le parent

## Resume

OpenCode separe le demarrage d'un sous-agent et la reception de son resultat final.
Avec un mode detache, le parent recoit tout de suite un identifiant, continue son travail, puis recupere le rapport plus tard.

CL-GO-DASH lance deja les sous-agents en sessions enfants et peut en lancer plusieurs en parallele.
Le comportement bloquant reste disponible avec `mode: "wait"`, mais le nouveau pilotage ajoute `mode: "detach"` pour rendre la main au parent.

## Objectif produit

Tu transformes les sous-agents en vrais jobs pilotables par le parent.
Le parent doit pouvoir :

- lancer un sous-agent en mode detache ;
- continuer a utiliser d'autres tools pendant que le sous-agent travaille ;
- voir quels sous-agents sont actifs ;
- lire l'etat ou le dernier avancement d'un sous-agent ;
- ajouter une consigne a un sous-agent existant ;
- attendre explicitement un ou plusieurs sous-agents ;
- annuler un sous-agent si besoin ;
- reutiliser plus tard une session enfant terminee.

## Nommage attendu

Le type technique reste `explorer` ou `coder`, mais l'UI ne doit pas se limiter a ce type.
Chaque sous-agent doit avoir deux champs separes :

- `display_name` : nom court affiche dans l'UI.
- `description` : resume de mission, par exemple `Analyse sous-agent repo Claude Code`.

Valeurs par defaut verrouillees :

- `coder` : `Claudiator`, couleur `--subagent-claudiator` orange.
- `explorer` : `Geminitor`, couleur `--subagent-geminitor` bleu clair.

Affichage cible :

```text
Jackson
Analyse sous-agent repo Claude Code
```

Le parent peut fournir `display_name` et `description` dans `delegate_task`.
S'il ne les fournit pas, CL-GO-DASH applique les valeurs par defaut ci-dessus.

## Reference OpenCode

OpenCode expose un tool `task` avec `description`, `prompt`, `subagent_type`, `task_id` et `background`.
Quand `background=true`, le tool retourne rapidement un resultat `state="running"` au parent.
Le parent peut donc continuer son tour au lieu d'attendre la fin du sous-agent.

OpenCode possede aussi une couche interne de jobs avec ces actions :

- `list`
- `get`
- `start`
- `extend`
- `wait`
- `promote`
- `cancel`

Le point important n'est pas de copier OpenCode tel quel.
Le point important est la separation entre :

- demarrer le sous-agent ;
- suivre son etat ;
- attendre son resultat ;
- injecter son rapport final plus tard.

## Etat actuel CL-GO-DASH

Le tool actuel est `delegate_task`.
Il accepte `prompt`, `subagent_type` et `name`.

Le backend cree une session enfant avec :

- `parent_session_id`
- `subagent_type`
- `subagent_prompt`
- `subagent_status`
- `subagent_run_id`

Le sous-agent tourne bien dans une tache async.
Mais le parent attend le `oneshot` de completion via `PendingDelegate.wait()`.

Plusieurs appels `delegate_task` consecutifs sont bien lances en parallele, mais le parent ne reprend la main qu'une fois les rapports recus.

## Ecarts principaux

| Capacite | OpenCode | CL-GO-DASH actuel | Cible CL-GO-DASH |
|---|---|---|---|
| Continuer pendant un sous-agent actif | Oui avec `background=true` | Non | Oui |
| Recevoir un ID reutilisable | Oui, `task_id` | Pas expose au LLM | Oui, `subagent_id` |
| Reprendre une session enfant | Oui | Non | Oui |
| Ajouter une consigne a un sous-agent actif | Oui via `extend` | Non | Oui |
| Inspecter les sous-agents actifs | Service interne/UI | UI seulement | Tool LLM + UI |
| Annuler un sous-agent | Service interne/UI | UI seulement | Tool LLM + UI |
| Rapport final plus tard | Oui, message synthetique | Non, resultat bloquant | Oui |
| Nom + description UI | Oui via description/titre | Nom seulement | Oui |

## Architecture cible

### Registre de jobs

Tu ajoutes une couche `SubagentJob` au-dessus des sessions enfants.
Elle doit stocker au minimum :

- `subagent_id`
- `parent_session_id`
- `child_session_id`
- `display_name`
- `description`
- `subagent_type`
- `status`
- `created_at`
- `updated_at`
- `completed_at`
- `summary`
- `last_activity`
- `error_kind`

Le registre memoire reste borne.
Les donnees importantes doivent aussi etre persistables via les sessions existantes pour eviter de perdre l'etat au redemarrage.

### Tools exposes au parent

Tu gardes `delegate_task`, mais tu ajoutes le mode detache :

```json
{
  "prompt": "...",
  "subagent_type": "explorer",
  "display_name": "Geminitor",
  "description": "Analyse sous-agent repo Claude Code",
  "mode": "detach"
}
```

Resultat immediat attendu :

```xml
<subagent id="..." state="running">
Sous-agent lance en session enfant.
</subagent>
```

Tu ajoutes ensuite des tools explicites :

- `list_subagents` : liste courte des sous-agents du parent.
- `get_subagent` : detail d'un sous-agent et dernier etat connu.
- `wait_subagent` : attend un ou plusieurs sous-agents.
- `cancel_subagent` : annule un sous-agent actif.
- `message_subagent` : ajoute une consigne a une session enfant existante.

Ces tools doivent retourner des messages courts et generiques en cas d'erreur.
Les details internes restent uniquement dans les logs.

### Rapport final differe

Quand un sous-agent detache termine, CL-GO-DASH ajoute un rapport cache borne dans la session parent.
Ce rapport n'apparait pas comme message visible dans le chat parent.
Il est injecte une seule fois dans le contexte LLM du parent au prochain tour.

Le parent doit recevoir plus tard :

```xml
<subagent id="..." state="completed">
<summary>...</summary>
</subagent>
```

Si le parent est encore en stream, il peut continuer naturellement.
Si le parent n'est plus en stream, le resultat reste dans la session et pourra etre consomme au prochain tour.

### Compatibilite avec le flux actuel

Tu conserves le comportement bloquant par defaut pour eviter une rupture brutale :

- `mode` absent ou `mode: "wait"` : `delegate_task` attend le rapport comme aujourd'hui.
- `mode: "detach"` : `delegate_task` retourne immediatement un ID.

## Priorites

### P0

- Tu ajoutes `mode: "wait" | "detach"` a `delegate_task`.
- Tu ajoutes `display_name` et `description`.
- Tu exposes un `subagent_id` stable au parent.
- Tu crees le registre `SubagentJob`.
- Tu garantis que le parent peut continuer son stream apres un lancement detache.

### P1

- Tu ajoutes `list_subagents`, `get_subagent`, `wait_subagent`, `cancel_subagent`.
- Tu ajoutes l'injection du rapport final differe dans le contexte parent.
- Tu permets de reutiliser une session enfant avec `subagent_id`.

### P2

- Tu ajoutes `message_subagent` pour pousser une nouvelle consigne a un sous-agent actif ou termine.
- Tu appliques les profils par defaut : `Claudiator` pour `coder`, `Geminitor` pour `explorer`.
- Tu ajoutes une vue UI plus claire avec nom, description, statut et dernier evenement.

## Points de vigilance

- Ne laisse pas une collection de jobs grossir sans limite.
- N'expose pas les erreurs internes au parent ou a l'utilisateur.
- Annule les sous-agents enfants quand le parent est annule.
- Ne reinjecte pas deux fois le meme rapport final.
- Garde l'ordre logique des rapports quand plusieurs sous-agents terminent en ordre inverse.
- Marque les jobs actifs comme `interrupted` au redemarrage si le process a ete ferme.
- Evite qu'un sous-agent termine reste indefiniment considere comme actif.

## Questions a trancher

- Les noms par defaut sont codes pour cette iteration : `Claudiator` et `Geminitor`.
- Le rapport final differe est cache comme contexte technique pour le parent.
- Un sous-agent `coder` reutilise doit-il garder le meme worktree ou repartir dans un nouveau worktree ?
- Le parent peut annuler un sous-agent sans confirmation utilisateur.
- Faut-il autoriser un sous-agent a lancer lui-meme d'autres sous-agents ?

## Fichiers references

- `/Users/kevinh/Projects/analyse-repo/opencode/packages/opencode/src/tool/task.ts`
- `/Users/kevinh/Projects/analyse-repo/opencode/packages/core/src/background-job.ts`
- `/Users/kevinh/Projects/CL-GO-DASH/src-tauri/src/services/agent_local/tool_definitions_subagent.rs`
- `/Users/kevinh/Projects/CL-GO-DASH/src-tauri/src/services/agent_local/tool_dispatcher_delegate.rs`
- `/Users/kevinh/Projects/CL-GO-DASH/src-tauri/src/services/agent_local/tool_executor_delegate_batch.rs`
- `/Users/kevinh/Projects/CL-GO-DASH/src-tauri/src/services/agent_local/subagent_task.rs`
- `/Users/kevinh/Projects/CL-GO-DASH/src-tauri/src/services/agent_local/subagent_registry.rs`
- `/Users/kevinh/Projects/CL-GO-DASH/src-tauri/src/commands/subagents.rs`
