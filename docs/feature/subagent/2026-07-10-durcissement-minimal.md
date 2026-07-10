# Durcissement minimal des sous-agents

## Statut du document

Ce document constitue le contrat d'implémentation de la branche
`codex/subagent-hardening-minimal`.

- Pars exclusivement de `main` au commit `7fe28426`.
- Conserve `codex/subagent-runtime-hardening` comme archive rejetée.
- Ne cherry-picke aucun commit complet de la branche rejetée.
- Réimplémente uniquement les changements autorisés ci-dessous.
- Arrête le travail si un besoin impose de dépasser ce contrat.
- Demande une nouvelle approbation utilisateur avant d'élargir le périmètre.

Les documents suivants restent des références historiques. Ne les utilise pas
comme plans d'implémentation pour cette branche :

- `docs/feature/subagent/2026-07-07-pilotage-parent.md`
- `docs/feature/sous-agent.md`

## Baseline réelle de `main`

Ne considère plus le flux sous-agent de `main` comme une boîte noire
fonctionnelle.

Le test naturel de la session
`1a61e975-5e82-48c7-ac61-a28cc926d201` sur `main` `7fe28426` a reproduit les
régressions observées sur la branche rejetée :

- appels répétés de `list_subagents` et `get_subagent` toutes les deux à
  quatre secondes ;
- messages de finalisation envoyés prématurément aux enfants ;
- annulations puis redélégations inutiles ;
- consommation de tours modèle sans changement réel d'état ;
- synthèse parent prématurée pendant la phase de travail.

La cause confirmée se trouve dans `main` : après chaque outil de contrôle, la
boucle rappelle immédiatement le modèle et réinjecte un message qui lui propose
d'inspecter, attendre, annuler ou relancer les enfants. Le waiter sain n'est
atteint que lorsque le modèle produit enfin un tour sans outil, puis il provoque
encore un premier rappel immédiat.

Corrige uniquement cette mécanique et les courses terminales associées. Ne
profite pas de cette exception pour réécrire l'architecture agentique.

## Objectif

Reproduis le principe utile de Claude Code sans copier son système complet :

- laisse le parent poursuivre un travail réellement indépendant après
  `delegate_task` ;
- suspends ensuite les appels LLM sans délai périodique ;
- réveille le parent uniquement lors d'un rapport, d'un échec terminal, d'un
  nouveau message utilisateur ou de l'annulation du stream ;
- livre une correction à l'enfant actif au prochain tour interne, sans créer
  un second run ;
- conserve les rapports complets et leur sécurité ;
- durcis ensuite les outils, les prompts, les worktrees, les permissions et
  les modes sans nouvelle modification du flux.

Ne copie ni la mailbox générale, ni les teammates, ni le processeur global de
notifications de Claude Code. Conserve le stream parallèle existant et ajoute
seulement une attente événementielle ciblée.

## Orchestration parent événementielle

### Guidance unique du parent

Après `delegate_task`, autorise un nouveau tour modèle afin que le parent puisse
continuer son propre travail.

Quand aucun travail utile ne reste :

- demande au parent d'informer brièvement l'utilisateur une seule fois ;
- demande-lui de rendre un tour sans outil ;
- indique que le runtime attendra et injectera automatiquement les rapports ;
- interdis-lui de sonder les enfants pour attendre ;
- réserve `message_subagent` à une correction concrète ;
- réserve `cancel_subagent` à une demande utilisateur ou à une direction
  manifestement incorrecte.

Retire les formulations techniques ou anxiogènes comme :

- `Final answer is locked` ;
- `blocked` ;
- `Keep the stream active` ;
- les invitations à `inspect/wait/cancel/message` en continu.

Ne génère aucun texte conversationnel depuis le backend. Le LLM reste l'auteur
des informations visibles par l'utilisateur.

### Attente sans appel LLM

Supprime :

- le premier rappel immédiat ;
- le rappel toutes les dix minutes ;
- le polling interne d'une seconde.

Utilise un `tokio::sync::watch` partagé par les enfants actifs d'un même parent.
Ne crée aucun registre non borné : rattache le signal aux entrées déjà limitées
à quatre enfants par parent et huit enfants au total.

Souscris au signal avant de vérifier les rapports et l'état actif afin de ne
perdre aucun réveil. Attends ensuite uniquement :

- un changement terminal signalé ;
- l'annulation du token du stream.

Ne réveille jamais le parent lors d'une activité ordinaire, d'un appel d'outil
enfant ou d'une simple mise à jour UI.

### Contrôles purs

Centralise une fonction pure `is_control_only()` utilisée par les deux boucles
API et Ollama.

Classe comme contrôles :

- `list_subagents` ;
- `get_subagent` ;
- `message_subagent` ;
- `cancel_subagent` ;
- `archive_subagent`.

Exclus toujours `delegate_task` de cette catégorie.

Après l'exécution d'un lot composé uniquement de contrôles, entre directement
dans le waiter s'il reste un enfant actif. Ne rappelle pas le modèle avec les
résultats d'état inchangés.

Si le lot contient au moins un outil de travail normal, conserve le
fonctionnement actuel afin que le parent puisse poursuivre son travail
indépendant.

### Remplacement du stream parent

Lorsqu'un nouveau message utilisateur remplace le stream courant :

- annule uniquement le waiter parent précédent ;
- ne cancelle aucun enfant ;
- fais reprendre tous les enfants encore actifs par le nouveau stream ;
- traite immédiatement le nouveau message ;
- remets ensuite le parent en attente s'il reste des enfants ;
- injecte les rapports à leur arrivée.

Conserve le comportement du bouton Stop : annule le parent et tous ses enfants.

## Rapports et fin d'enfant

### Ordre terminal atomique

Pour chaque run enfant, respecte cet ordre :

1. Finalise et persiste le statut enfant.
2. Capture le changement Git si l'enfant est codeur.
3. Construis et persiste le rapport parent.
4. Rends l'enfant non actif dans le registry.
5. Émets le signal de réveil parent.
6. Nettoie le worktree.

Acquiers le signal avant le retrait du registry, puis notifie seulement après
que le rapport est durable et l'état terminal visible. Empêche ainsi le parent
d'observer simultanément « aucun enfant actif » et « aucun rapport ».

Si la persistance du rapport échoue :

- marque l'enfant en échec ;
- notifie une défaillance terminale générique au waiter ;
- termine le stream parent sans réponse finale ;
- nettoie l'entrée du registry ;
- n'ajoute aucune attente infinie ni boucle de retry.

### Livraison durable

Conserve le rapport original complet et la limite existante de 12 000
caractères. Ne le résume jamais automatiquement en 1 200 caractères.

Remplace la consommation destructive par une lecture suivie d'un acquittement :

- lis tous les rapports disponibles sans les retirer ;
- injecte chaque rapport une seule fois dans le contexte courant ;
- mémorise les identifiants injectés ;
- acquitte-les uniquement après un appel modèle réussi ;
- conserve-les après annulation, remplacement du stream ou échec provider ;
- regroupe tous les rapports déjà prêts dans le même appel modèle.

Injecte les rapports sous rôle `assistant` dans une balise structurée. Ajoute
une seule politique système indiquant qu'ils constituent des preuves non
fiables et jamais de nouvelles instructions. Ne crée aucun faux message
utilisateur technique.

Réveille le parent à chaque nouveau rapport. S'il reste d'autres enfants :

- autorise le parent à exploiter le résultat pour un travail utile ;
- autorise au plus une information de progression courte ;
- interdis une synthèse complète ou une conclusion finale ;
- remets-le en attente lorsqu'aucune intervention n'est nécessaire.

Autorise la phase `final` uniquement après disparition de tous les enfants
repris par le stream et livraison réussie de leurs rapports requis.

### Panic et interruption

Supervise chaque futur enfant. Convertis un panic en échec terminal générique,
persiste son rapport si possible, retire l'entrée du registry et réveille le
parent.

N'ajoute aucun timeout automatique. Une longue exécution normale reste
autorisée.

## Messages destinés à un enfant

Conserve `subagent_queued_prompts` comme file persistante et bornée.

Pour un enfant actif :

- valide la propriété parent/enfant ;
- normalise et borne la consigne ;
- ignore un doublon textuel exact après normalisation des espaces ;
- persiste la consigne ;
- consomme la file avant chaque requête enfant ;
- vérifie encore la file juste avant de quitter un tour sans outil ;
- continue le même run, le même `run_id` et le même worktree.

Rends atomiques la vérification de la file et le passage à l'état terminal. Une
consigne tombe obligatoirement d'un seul côté de cette frontière :

- avant la fermeture, injecte-la au prochain tour interne ;
- après une fin réussie, crée une reprise explicite ;
- après annulation, échec ou interruption, refuse-la et exige un nouveau
  `delegate_task` avec `subagent_id`.

Ne crée ni canal de messages non persistant, ni mailbox générale, ni messages
entre enfants.

## Profils d'outils

Crée une source de vérité backend `SubagentToolProfile`. Utilise-la pour :

- construire les définitions exposées au modèle ;
- décrire ces outils dans le prompt enfant ;
- valider chaque appel avant autorisation et exécution.

Refuse tout outil hors profil avant la demande d'autorisation. Échoue fermé si
une session enfant ne possède plus de `subagent_type` valide.

### Explorateur

Expose uniquement :

- `bash` ;
- `read_file` ;
- `list_dir` ;
- `grep` ;
- `glob` ;
- `web_search` ;
- `web_fetch`.

Exécute le Bash explorateur sans shell intermédiaire. Autorise uniquement :

- `pwd` ;
- `ls` ;
- `tree`, avec `-L` suivi d'une profondeur comprise entre 1 et 8 ;
- `file` ;
- `stat` ;
- `wc` ;
- `du` ;
- `df` ;
- les formes de lecture prévues pour `git status`, `git diff`, `git log`,
  `git show`, `git rev-parse`, `git ls-files`, `git remote -v`,
  `git tag --list` et `git branch` sans mutation.

Refuse `find`, le réseau, les mutations, les pipes, les séparateurs, les
redirections, les sous-shells et tout chemin sortant du dossier autorisé.
Recommande `list_dir` ou `glob` à la place de `find`.

Ne présente jamais les skills ou les outils indisponibles à l'explorateur.

### Codeur

Expose uniquement :

- `bash` ;
- `read_file` ;
- `write_file` ;
- `edit_file` ;
- `list_dir` ;
- `grep` ;
- `glob` ;
- `web_search` ;
- `web_fetch` ;
- `load_skill` uniquement si les skills sont activées.

Ne lui expose jamais Forecast, Office, MCP, image, Plan mode, todo, choix
utilisateur, délégation ou contrôle de sous-agent.

Quand `load_skill` est disponible, présente uniquement la liste réellement
accessible dans les réglages. Ne révèle aucun skill désactivé.

Limite les outils fichiers au worktree canonique. Refuse les chemins absolus
extérieurs, les traversées `..` et les symlinks sortants.

Ajoute une protection Bash pragmatique contre les sorties explicites vers le
dépôt parent ou un autre dépôt. Conserve les commandes usuelles de test, build
et Git dans le worktree. Ne présente pas cette protection comme une sandbox du
système d'exploitation.

## Prompts et contexte enfant

Conserve Claudiator et Geminitor comme simples surnoms. Utilise toujours le
modèle et le provider du parent.

Sépare l'identité enfant du mode de permission avec
`StreamTaskParams.subagent_profile`.

Fais hériter exactement :

- `thinking_enabled` ;
- `reasoning_mode` ;
- le mode manuel ou automatique ;
- la langue de réponse configurée.

Injecte une seule fois les instructions `AGENTS.md` applicables et la
personnalité activée dans chaque enfant, explorateur compris.

Résous le véritable worktree avant de composer le prompt codeur. Injecte son
chemin comme dossier d'exécution autoritaire.

Mutualise les règles métier du parent et du codeur :

- explore avant d'écrire ;
- cherche l'implémentation existante ;
- respecte `AGENTS.md` ;
- limite le changement au besoin ;
- traite ou propage les erreurs ;
- teste le changement ;
- vérifie le diff final.

Utilise le réglage de langue comme unique autorité linguistique, titres de
rapport compris.

Guide les rapports :

- Explorateur : conclusion, constats confirmés, preuves, incertitudes.
- Codeur : résultat, changements, fichiers, vérifications, risques.

## Worktrees éphémères

Pour chaque exécution ou réexécution du codeur :

- pars du HEAD parent courant ;
- crée une branche `cl-go/subagent/<uuid>` ;
- crée un nouveau worktree géré par l'application ;
- ne réutilise jamais un ancien worktree ;
- crée automatiquement un commit avec tous les changements ;
- persiste un `SubagentChangeMeta` borné ;
- supprime immédiatement le worktree après la capture.

Persiste au minimum :

- l'identifiant du changement ;
- l'identifiant du projet ;
- le commit de départ ;
- le commit produit ;
- la branche temporaire ;
- la branche cible ;
- les fichiers ajoutés, modifiés, supprimés ou renommés ;
- le statut du changement ;
- les dates de création et de mise à jour.

Borne le nombre de changements conservés et le nombre de chemins enregistrés
par changement.

Conserve uniquement la branche légère tant que le changement attend une
décision.

Expose uniquement au parent :

- `inspect_subagent_changes` ;
- `apply_subagent_changes` ;
- `discard_subagent_changes`.

Intègre avec Git trois voies. En cas de conflit :

- annule l'opération ;
- restaure exactement l'état initial du dépôt parent ;
- conserve la branche temporaire ;
- retourne un diagnostic borné ;
- ne lance aucune boucle automatique.

Lors d'une correction, repars du nouveau HEAD parent et rejoue le commit encore
en attente dans un nouveau worktree. Conserve l'ancien changement intact si le
rejeu échoue.

Après interruption ou crash, capture les changements dans un commit temporaire
si possible, marque le run interrompu puis supprime le worktree. Au démarrage,
inspecte uniquement le dossier de worktrees géré par l'application, récupère ou
abandonne chaque entrée orpheline de façon bornée, puis nettoie-la. Ne conserve
jamais un worktree à long terme.

En mode manuel, demande l'autorisation avant `apply_subagent_changes`. En mode
automatique, conserve le fonctionnement sans demande voulu par le produit.

Ne bloque pas la réponse finale uniquement parce qu'une branche temporaire
reste en attente. Guide cependant le parent pour qu'il inspecte, applique ou
abandonne avant d'affirmer que le code est intégré.

## Permissions enfant

Ajoute :

```text
StreamTaskParams.permission_emitter: Option<AgentEventEmitter>
```

- Utilise l'émetteur courant pour une session normale.
- Utilise l'émetteur parent pour une session enfant.
- Exécute toujours l'outil sous l'identité enfant.
- Enregistre `AllowSession` pour l'enfant.
- Ne change aucun événement Tauri public ni aucune commande de réponse.
- Transmets le paramètre dans API et Ollama sans changer leur ordre métier.

## Verrouillage du mode

Persiste `PermissionFamily::{Chat, Tools}`.

Lors du premier envoi :

- verrouille Chatbot sur `Chat` ;
- verrouille Demander l'autorisation ou Accès complet sur `Tools` ;
- autorise ensuite uniquement le basculement entre ces deux modes outillés.

Applique la règle aux anciennes sessions lors de leur prochain envoi. Fais-la
respecter dans le backend et l'interface. Affiche toujours le vrai libellé et
n'affiche jamais `Agent` comme nom de mode.

## Dossier de session disparu

Vérifie le dossier juste avant l'envoi.

S'il manque :

- suspends uniquement l'envoi courant ;
- retourne depuis le backend un état structuré contenant au minimum le chemin
  manquant et le parent existant le plus proche ;
- affiche le chemin attendu et le parent existant le plus proche ;
- propose `Switcher` et `Créer` près du sélecteur ;
- mets à jour la session avec `Switcher` ;
- recrée uniquement le chemin vide avec `Créer` ;
- rejoue une seule fois le même envoi après succès ;
- empêche tout doublon du message utilisateur.

Ne restaure aucun fichier. N'affiche aucune erreur technique brute. Ajoute les
textes visibles dans les sept langues prises en charge. Ne demande jamais au
frontend d'interpréter une erreur textuelle pour reconstruire cet état.

## Zones protégées

En dehors des exceptions explicitement décrites, ne modifie pas :

- la sémantique générale des boucles API et Ollama ;
- l'ordre des outils normaux ;
- les retries provider et la compression ;
- le rendu des phases `work` et `final` hors orchestration ;
- les événements Tauri publics ;
- les messages en file des sessions parentes ;
- le fonctionnement du bouton Stop ;
- les domaines Forecast, Office, MCP, image, Plan et todo.

Dans les boucles modèle, ajoute uniquement les petits points d'intégration
partagés pour l'attente, l'acquittement, les messages live et l'émetteur de
permission. Ne duplique aucune logique métier entre API et Ollama.

## Changements interdits

N'ajoute pas :

- une machine d'état parent/enfant générale ;
- un checkup initial ou périodique ;
- un timeout automatique des enfants ;
- une mailbox globale ou des teammates ;
- un texte conversationnel généré par le backend ;
- un résumé destructeur des rapports ;
- une boucle automatique de résolution des conflits Git ;
- une conservation longue durée des worktrees ;
- une sandbox OS présentée comme garantie.

Ne corrige aucun problème théorique extérieur à ce contrat. Documente-le et
demande une décision utilisateur.

## Leçons de la branche rejetée

La branche `codex/subagent-runtime-hardening` a touché 197 fichiers avec environ
9 987 ajouts et 1 675 suppressions. Ce volume a masqué une réécriture du flux et
rendu les régressions difficiles à isoler.

Ne reproduis jamais ces erreurs :

- ne transforme pas une guidance de prompt en boucle runtime ;
- ne rappelle pas le modèle sans événement réel ;
- ne force pas un enfant à terminer parce qu'il travaille encore ;
- ne confonds pas reclassification en `work` et prévention réelle d'une fin ;
- ne retire pas un rapport avant un appel modèle réussi ;
- ne rends pas l'enfant inactif avant la persistance du rapport ;
- ne lance pas un nouveau run pour une correction livrable au tour suivant ;
- ne mélange pas plusieurs domaines dans un même commit ;
- ne considère pas les seuls tests structurels comme preuve du comportement
  naturel d'un LLM.

## Stratégie de commits et portes

Utilise cet ordre :

1. `Revise minimal subagent hardening scope`
2. `Make subagent completion atomic`
3. `Stop parent subagent polling`
4. `Deliver live subagent corrections`

Après le quatrième commit, exécute les validations du jalon orchestration puis
arrête-toi pour le test UI naturel de Kevin.

Après son approbation explicite, poursuis :

5. `Harden subagent tool profiles`
6. `Align subagent prompts and inherited context`
7. `Add ephemeral coder worktree lifecycle`
8. `Route child permissions to parent stream`
9. `Lock session modes without changing agent flow`
10. `Recover missing session directories`

Avant chaque commit :

- affiche `git diff --stat` ;
- liste les fichiers modifiés ;
- vérifie leur appartenance au lot ;
- vérifie les zones protégées ;
- inclus dans chaque commit de code les tests couvrant le comportement ajouté
  ou modifié ;
- exécute les tests ciblés ;
- arrête et redécoupe le lot s'il dépasse 25 fichiers ou environ 1 500 lignes
  modifiées ;
- maintiens les fichiers modifiés sous les limites du projet sans compacter le
  code ni lancer de refactor annexe.

## Validation du jalon orchestration

Ajoute des tests déterministes qui prouvent :

- la poursuite du travail parent après `delegate_task` ;
- l'attente immédiate après un tour sans outil ;
- l'absence de tout réveil après une heure simulée sans événement ;
- la mise en attente après un lot de contrôles purs ;
- la poursuite après un contrôle accompagné d'un outil utile ;
- la parité API et Ollama ;
- le réveil unique à chaque rapport ;
- le regroupement de rapports simultanés ;
- la persistance avant retrait du registry ;
- l'acquittement uniquement après succès modèle ;
- l'absence de faux message utilisateur ;
- la politique système anti-injection unique ;
- le nettoyage après panic, annulation et échec ;
- la correction live dans le même run ;
- la course terminale sans perte ni doublon ;
- la reprise des enfants par un nouveau stream utilisateur.

Exécute ensuite un test UI naturel avec deux explorateurs sans annoncer au LLM
qu'il s'agit d'un test interne. Vérifie :

- lancement parallèle ;
- travail parent utile ;
- au plus une information courte avant l'attente ;
- zéro appel LLM pendant trois minutes sans événement ;
- aucun `list/get/message/cancel` automatique ;
- réveil au premier rapport sans synthèse complète ;
- attente du second enfant ;
- synthèse finale après le dernier rapport ;
- envoi d'un nouveau message pendant l'attente sans annulation des enfants,
  puis livraison de leurs rapports dans le nouveau stream ;
- absence d'interruption ou de rapport forcé.

## Validation des lots suivants

Ajoute les tests ciblés pour :

- l'égalité exacte entre outils décrits, exposés et exécutables ;
- le refus des outils retirés avant autorisation ;
- le Bash explorateur autorisé et interdit ;
- le confinement du codeur ;
- l'héritage modèle, provider, thinking, langue, personnalité et `AGENTS.md` ;
- le cycle Git ajout, modification, suppression, commit, nettoyage,
  application, abandon, conflit et reprise ;
- la permission enfant reçue dans le parent et appliquée à l'enfant ;
- le verrouillage des familles de mode ;
- les actions `Switcher` et `Créer`.

Termine la branche par :

- les tests Rust ciblés ;
- `cargo test` ;
- `cargo clippy --all-targets` ;
- `cargo check` ;
- les tests frontend ciblés ;
- `npm test` ;
- `npx tsc --noEmit` ;
- la mise à jour Graphify ;
- le contrôle final du diff contre `main`.

Ne pousse rien et ne crée ni merge ni pull request sans demande explicite.

## Porte d'approbation

Après le commit de cette révision, arrête le travail. Demande à Kevin de relire
ce document et de l'approuver. Ne modifie aucun fichier de code avant cette
approbation explicite.
