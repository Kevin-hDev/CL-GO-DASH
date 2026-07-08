# Pilotage des sous-agents par le parent

## Resume

OpenCode separe le demarrage d'un sous-agent et la reception de son resultat final.
Le parent recoit un identifiant, peut continuer son travail en parallele, mais l'orchestrateur garde le controle tant que les sous-agents actifs n'ont pas termine.

CL-GO-DASH lance deja les sous-agents en sessions enfants et peut en lancer plusieurs en parallele.
La cible n'est plus d'avoir deux modes `wait` et `detach`.
La cible est un seul comportement : le parent delegue, continue ce qu'il peut faire, puis reste actif en attente jusqu'a reception des rapports.

## Objectif produit

Tu transformes les sous-agents en vrais jobs pilotables par le parent.
Le parent doit pouvoir :

- lancer un sous-agent pilotable ;
- continuer a utiliser d'autres tools pendant que le sous-agent travaille ;
- voir quels sous-agents sont actifs ;
- lire l'etat ou le dernier avancement d'un sous-agent ;
- ajouter une consigne a un sous-agent existant ;
- attendre explicitement un ou plusieurs sous-agents ;
- annuler un sous-agent si besoin ;
- reutiliser plus tard une session enfant terminee.

Le parent ne doit pas produire de reponse finale utilisateur tant qu'un sous-agent du tour courant est actif.
Il peut envoyer des messages courts d'avancement, mais la synthese finale commence seulement apres reception des rapports attendus.

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

## Etat avant refonte CL-GO-DASH

Avant cette refonte, le tool etait `delegate_task`.
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

### Version ideale : separer session enfant et runs

La version la plus propre a long terme est de ne pas confondre :

- la session enfant, qui est l'espace durable et consultable par l'utilisateur ;
- le run, qui est une execution precise d'une consigne dans cette session.

Aujourd'hui, CL-GO-DASH stocke surtout un statut global sur la session enfant via `subagent_status` et `subagent_run_id`.
Cela fonctionne, mais devient ambigu quand `message_subagent` ajoute une consigne en file :

```text
session enfant A
run 1 termine
run 2 demarre automatiquement
```

Dans ce cas, dire simplement `session A completed` puis `session A running` est difficile a comprendre.
Le correctif actuel garde la session en `running` tant qu'un run suivant est en file.
C'est coherent avec le modele actuel, mais ce n'est pas le modele ideal.

Modele cible :

```text
SubagentSession
  id
  parent_session_id
  name
  description
  subagent_type
  aggregate_status
  last_activity
  current_run_id
  run_history[]

SubagentRun
  id
  child_session_id
  parent_session_id
  prompt
  status
  started_at
  completed_at
  summary
  hidden_report_id
  queued_by
```

Regles cible :

- `SubagentSession.aggregate_status` reste `running` si au moins un run est actif ou en file.
- Un run termine peut etre `completed` sans faire passer la session a `completed` si un autre run doit demarrer.
- `wait_subagent` attend la session complete par defaut, pas seulement le run courant.
- `wait_subagent` peut accepter plus tard un `run_id` pour attendre un run precis.
- `get_subagent` affiche l'etat global de la session et le run courant.
- `list_subagents` affiche l'etat global de la session, pas uniquement le dernier run termine.
- Les rapports caches sont lies a un run, puis agreges dans la session.
- L'UI peut montrer une session enfant stable avec un historique de runs repliable.

Exemple ideal :

```xml
<subagent id="A" status="running" current_run_id="run-2">
  <run id="run-1" status="completed">
    <summary>Premier sleep termine.</summary>
  </run>
  <run id="run-2" status="running">
    <last_activity>sleep 30</last_activity>
  </run>
</subagent>
```

Ce modele supprime l'ambiguite `completed -> running` sur une meme session.
Il permet aussi au parent de raisonner clairement : un run peut etre termine, mais le sous-agent reste actif tant que la file n'est pas vide.

### Tools exposes au parent

Tu gardes `delegate_task`, mais tu ne exposes pas deux modes au LLM.
`delegate_task` lance toujours un sous-agent pilotable :

```json
{
  "prompt": "...",
  "subagent_type": "explorer",
  "display_name": "Geminitor",
  "description": "Analyse sous-agent repo Claude Code"
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

Ces tools appartiennent tous a la meme feature que `delegate_task`.
Cote settings, l'utilisateur active/desactive la feature "Sous-agents" comme un seul groupe logique.
Si `delegate_task` est actif, les tools de pilotage doivent etre disponibles aussi.
Si `delegate_task` est desactive, tous les tools sous-agent sont desactives et apparaissent seulement dans la liste courte des tools indisponibles.

### Rapport final differe

Quand un sous-agent termine, CL-GO-DASH ajoute un rapport cache borne dans la session parent.
Ce rapport n'apparait pas comme message visible dans le chat parent.
Il est injecte une seule fois dans le contexte LLM du parent pendant le stream courant, au prochain point sur de reprise du parent.
Un reveil automatique au prochain tour reste seulement un filet de securite si le stream est coupe par erreur.

Le parent doit recevoir plus tard :

```xml
<subagent id="..." state="completed">
<summary>...</summary>
</subagent>
```

Le stream parent doit rester actif tant qu'un sous-agent du tour courant est actif.
Le backend ne doit pas laisser le parent cloturer definitivement avant reception des rapports attendus.

### Rapports pendant que le parent travaille

Un rapport de sous-agent peut arriver pendant que le parent est deja occupe :

- en train de recevoir une reponse modele ;
- en train d'executer un tool ;
- en train d'enchainer plusieurs actions utiles en parallele.

Dans ce cas, le rapport ne doit pas interrompre brutalement l'appel en cours.
Le rapport doit etre stocke comme contexte cache en attente.
Le parent doit le recevoir au prochain point sur, c'est-a-dire avant le prochain appel modele qui peut utiliser cette information.

Objectif produit :

- le parent n'est pas perturbe au milieu d'un appel ou d'un tool ;
- le rapport n'attend pas forcement la toute fin du travail parent ;
- si le rapport contient une information necessaire, le parent peut l'utiliser pour la suite de son travail ;
- le parent peut informer brievement l'utilisateur qu'un sous-agent a termine, puis continuer son propre travail ;
- la reponse finale reste interdite tant que tous les sous-agents requis ne sont pas termines.

Comportement cible :

```text
Parent travaille en parallele
Sous-agent A termine
Rapport A stocke
Parent arrive au prochain point sur
Rapport A injecte dans le contexte
Parent peut adapter la suite de son travail
```

Ce point est important : "rapport cache" ne veut pas dire "rapport injecte uniquement quand le parent pense avoir fini".
Le rapport doit etre disponible des que l'orchestrateur peut relancer proprement le modele parent.

### Plusieurs sous-agents

Quand plusieurs sous-agents sont lances dans le meme tour, leurs rapports peuvent arriver dans n'importe quel ordre.

Le parent doit pouvoir recevoir un rapport partiel sans etre autorise a faire la synthese finale.
Exemple cible :

```text
Le premier sous-agent a termine. Son retour est interessant.
Je vais maintenant verifier ou en est le second sous-agent et attendre son rapport avant la synthese finale.
```

Regles cible :

- chaque rapport termine est stocke une seule fois ;
- chaque rapport termine est injecte au parent au prochain point sur ;
- le parent peut faire un update court visible apres un rapport partiel ;
- les sous-agents encore actifs continuent de bloquer la reponse finale ;
- la vraie synthese finale commence seulement quand les rapports requis sont disponibles ;
- l'ordre de contexte doit rester coherent pour le parent meme si les sous-agents finissent en ordre inverse.

### Boucle d'orchestration parent

Le backend controle la boucle :

- le parent lance un ou plusieurs sous-agents ;
- le parent continue les actions qu'il peut faire en parallele ;
- si un rapport de sous-agent arrive pendant que le parent travaille, il est stocke puis injecte au prochain point sur ;
- si le parent arrive a une fin de tour alors que des sous-agents sont encore actifs, le backend garde le stream ouvert ;
- le backend injecte un rappel cache dans le contexte du parent ;
- le parent doit checker l'etat des sous-agents et informer brievement l'utilisateur de l'avancement ;
- le rappel est reinjecte toutes les 10 minutes tant qu'un sous-agent reste actif ;
- quand les rapports arrivent, ils sont injectes en contexte cache ;
- le parent peut alors synthétiser la reponse finale ou continuer si les rapports ouvrent une nouvelle piste utile.

### Phases de texte visibles

Tant qu'un sous-agent du tour courant reste actif, tout texte visible emis par le parent est un update de travail.
Ce texte ne doit pas etre classe comme reponse finale.

Regle cible :

- sous-agent actif : texte parent visible = phase `work` ;
- rapport partiel recu mais autre sous-agent actif : texte parent visible = phase `work` ;
- tous les rapports requis recus et plus aucun sous-agent actif : le parent peut enfin produire une phase `final`.

Le backend doit donc empecher une phase `final` prematuree.
Il doit aussi separer proprement les tours internes du stream pour eviter que l'UI melange :

- ancienne phase de travail ;
- update intermediaire ;
- vraie reponse finale.

Un `turnEnd` ou un equivalent de separation doit etre emis quand l'orchestrateur relance un tour interne.

Format cible du rappel cache :

```xml
<subagent_orchestration>
  <reason>subagents_still_running</reason>
  <instruction>
    Check active subagents, update the user briefly, then keep waiting.
    Do not produce the final answer until required subagent reports are available.
  </instruction>
  <active_subagents>
    <subagent id="..." name="Geminitor" status="running" elapsed="12m">
      <description>Analyse du repo</description>
      <last_activity>read_file src/...</last_activity>
    </subagent>
  </active_subagents>
</subagent_orchestration>
```

## Priorites

### P0

- Tu retires `mode: "wait" | "detach"` comme choix expose au LLM.
- Tu ajoutes `display_name` et `description`.
- Tu exposes un `subagent_id` stable au parent.
- Tu crees le registre `SubagentJob`.
- Tu garantis que le parent peut continuer son stream apres delegation.
- Tu garantis que le stream parent ne se ferme pas tant qu'un sous-agent du tour courant est actif.
- Tu garantis qu'aucun texte parent n'est marque `final` tant qu'un sous-agent du tour courant est actif.

### P1

- Tu ajoutes `list_subagents`, `get_subagent`, `wait_subagent`, `cancel_subagent`.
- Tu ajoutes l'injection du rapport final cache dans le contexte parent pendant le stream courant.
- Tu injectes les rapports termines au prochain point sur, pas uniquement quand le parent croit avoir fini.
- Tu permets de reutiliser une session enfant avec `subagent_id`.
- Tu ajoutes les rappels caches immediats puis toutes les 10 minutes.
- Tu corriges le groupe settings : `delegate_task` active/desactive tous les tools sous-agent.

### P2

- Tu ajoutes `message_subagent` pour pousser une nouvelle consigne a un sous-agent actif ou termine.
- Tu appliques les profils par defaut : `Claudiator` pour `coder`, `Geminitor` pour `explorer`.
- Tu ajoutes une vue UI plus claire avec nom, description, statut et dernier evenement.

## Points de vigilance

- Ne laisse pas une collection de jobs grossir sans limite.
- N'expose pas les erreurs internes au parent ou a l'utilisateur.
- Annule les sous-agents enfants quand le parent est annule.
- Ne laisse pas le parent envoyer une reponse finale tant qu'un sous-agent du tour courant est actif.
- Ne laisse pas l'UI afficher un update intermediaire comme une reponse finale.
- Ne reinjecte pas deux fois le meme rapport final.
- Les rappels d'attente ne doivent pas creer de messages visibles dans la conversation parent.
- Les messages d'avancement visibles du parent sont autorises, mais doivent rester courts.
- Garde l'ordre logique des rapports quand plusieurs sous-agents terminent en ordre inverse.
- Permets au parent d'utiliser un rapport partiel si ce rapport arrive pendant que d'autres sous-agents continuent.
- Marque les jobs actifs comme `interrupted` au redemarrage si le process a ete ferme.
- Evite qu'un sous-agent termine reste indefiniment considere comme actif.

## Etat observe le 2026-07-08

Le diagnostic a montre deux limites importantes dans l'implementation courante :

- la barriere sous-agent s'execute trop tard : elle garde le stream actif, mais seulement apres que le parent a deja pu produire un texte classe `final` ;
- les rapports caches sont consommes surtout quand la barriere s'execute apres un tour sans tool call, donc un rapport peut rester en attente si le parent continue a enchainer des tools.

Correction cible :

- introduire un vrai etat d'orchestration du tour courant ;
- savoir si des sous-agents du tour courant sont actifs avant de classifier le texte visible ;
- injecter les rapports termines avant le prochain appel modele possible ;
- garder les updates intermediaires en phase `work` ;
- n'autoriser la phase `final` qu'apres reception des rapports requis et absence de sous-agent actif.

## Questions a trancher

- Les noms par defaut sont codes pour cette iteration : `Claudiator` et `Geminitor`.
- Le rapport final differe est cache comme contexte technique pour le parent.
- Le rappel d'orchestration est cache comme contexte technique pour le parent.
- Le timeout de rappel est 10 minutes.
- Le reveil automatique apres fin de sous-agent sert uniquement de fallback, pas de mecanisme principal.
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
