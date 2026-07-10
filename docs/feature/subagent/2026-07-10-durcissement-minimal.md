# Durcissement minimal des sous-agents

## Statut du document

Ce document constitue le contrat d'implementation de la branche
`codex/subagent-hardening-minimal`.

- Pars exclusivement de `main` au commit `7fe28426`.
- Conserve `codex/subagent-runtime-hardening` comme archive rejetee.
- Ne cherry-picke aucun commit complet de la branche rejetee.
- Reimplemente uniquement les changements autorises ci-dessous.
- Arrete le travail si un besoin impose de modifier une zone protegee.
- Demande une nouvelle approbation utilisateur avant d'elargir ce contrat.

Les documents suivants restent des references historiques. Ne les utilise pas
comme plans d'implementation pour cette branche :

- `docs/feature/subagent/2026-07-07-pilotage-parent.md`
- `docs/feature/sous-agent.md`

## Objectif

Durcis les outils, les prompts, l'isolation Git, les permissions et les modes
de session sans modifier le comportement parent/enfant deja present sur
`main`.

Tu dois conserver exactement le fonctionnement existant pour :

- la poursuite du travail du parent apres une delegation ;
- l'attente des enfants ;
- les controles d'etat du parent ;
- les reveils lors des rapports ;
- les messages ajoutes a la file d'un enfant ;
- la transmission et l'injection des rapports ;
- la distinction visible entre phase de travail et reponse finale ;
- l'ordre des appels modele et des outils dans les boucles API et Ollama.

## Regle fondamentale

Considere le flux sous-agent de `main` comme une boite noire fonctionnelle.

Tu peux renforcer les donnees qui entrent dans ce flux et ajouter des
metadonnees factuelles a ses resultats. Tu ne changes jamais sa cadence, ses
conditions de sortie, sa logique d'attente ou sa facon de rappeler le modele.

Le routage des permissions constitue la seule exception autorisee dans les
boucles API et Ollama. Dans ce cas, ajoute uniquement un parametre et
transmets-le jusqu'au controle d'autorisation. Ne change aucune condition,
branche, boucle ou temporisation.

## Changements autorises

### Profils d'outils

Cree une source de verite backend unique pour chaque profil. Utilise-la pour :

- construire les definitions d'outils exposees au modele ;
- decrire ces outils dans le prompt enfant ;
- valider chaque appel avant autorisation et execution.

#### Explorateur

Expose uniquement :

- `bash` ;
- `read_file` ;
- `list_dir` ;
- `grep` ;
- `glob` ;
- `web_search` ;
- `web_fetch`.

Execute le Bash explorateur sans shell intermediaire. Autorise uniquement :

- `pwd` ;
- `ls` ;
- `tree`, avec `-L` suivi d'une profondeur comprise entre 1 et 8 ;
- `file` ;
- `stat` ;
- `wc` ;
- `du` ;
- `df` ;
- les formes de lecture deja definies pour `git status`, `git diff`,
  `git log`, `git show`, `git rev-parse`, `git ls-files`, `git remote -v`,
  `git tag --list` et `git branch` sans mutation.

Refuse :

- `find` ;
- les commandes reseau ;
- les mutations ;
- les pipes et separateurs ;
- les redirections ;
- les sous-shells et substitutions ;
- les chemins sortant du dossier autorise.

Recommande `list_dir` ou `glob` a la place de `find`. Ne presente jamais les
skills, les outils optionnels ou les outils desactives a l'explorateur.

#### Codeur

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
- `load_skill` si les skills sont activees dans les reglages.

Ne lui expose jamais Forecast, Office, MCP, image, Plan mode, todo, choix
utilisateur, delegation ou controle de sous-agent.

Limite les outils fichiers au worktree canonique. Refuse les chemins absolus
exterieurs, les traversees `..` et les symlinks qui sortent du worktree.

Ajoute une protection Bash pragmatique qui refuse les sorties explicites vers
le depot parent ou un autre depot. Conserve les commandes habituelles de test,
de build et de Git dans le worktree. Ne presente pas cette protection comme
une sandbox du systeme d'exploitation.

Refuse chaque outil retire avant toute demande d'autorisation.

### Prompts et contexte enfant

Conserve Claudiator et Geminitor comme surnoms. Utilise toujours le modele du
parent.

Herite exactement :

- du modele ;
- du provider ;
- de `thinking_enabled` ;
- de `reasoning_mode` ;
- du mode de permission.

Injecte une seule fois les instructions `AGENTS.md` applicables et la
personnalite activee dans chaque enfant, explorateur compris.

Utilise le reglage de langue comme unique autorite linguistique. Ne force
jamais des titres anglais si une autre langue est configuree.

Donne au codeur les memes regles metier que le parent :

- explore le code avant de le modifier ;
- cherche les implementations existantes ;
- respecte `AGENTS.md` ;
- limite la modification au besoin ;
- traite ou propage les erreurs ;
- teste le changement ;
- verifie le diff final.

Resous le worktree avant de construire le prompt codeur. Injecte son chemin
reel comme dossier d'execution autoritaire. Precise qu'un chemin mentionne
dans la mission ne remplace jamais ce dossier.

Guide le rapport sans imposer une taille courte :

- Explorateur : conclusion, constats confirmes, preuves, incertitudes.
- Codeur : resultat, changements, fichiers, verifications, risques.

Conserve le rapport original complet et la limite existante de 12 000
caracteres. Ne resume jamais automatiquement un rapport long en 1 200
caracteres.

Ne modifie ni le role, ni le moment, ni la persistance, ni la methode
d'injection des rapports de `main`.

### Worktrees ephemeres

Pour chaque execution ou reexecution du codeur :

- pars du HEAD courant de la branche parent ;
- cree une branche `cl-go/subagent/<uuid>` ;
- cree un nouveau worktree dans le dossier gere par l'application ;
- ne reutilise jamais un ancien worktree ;
- laisse le codeur travailler uniquement dans ce worktree ;
- cree automatiquement un commit temporaire avec tous les changements ;
- enregistre les metadonnees bornees ;
- supprime immediatement le worktree apres une capture reussie.

Conserve uniquement la branche Git legere tant que le changement attend une
decision.

Enregistre au minimum :

- l'identifiant du changement ;
- l'identifiant du projet ;
- le commit de depart ;
- le commit produit ;
- la branche temporaire ;
- la branche cible ;
- les fichiers ajoutes, modifies, supprimes ou renommes ;
- le statut du changement ;
- les dates de creation et de mise a jour.

Expose uniquement au parent :

- `inspect_subagent_changes` ;
- `apply_subagent_changes` ;
- `discard_subagent_changes`.

Integre un changement avec Git trois voies. En cas de conflit :

- annule l'operation ;
- restaure exactement l'etat initial du depot parent ;
- conserve la branche temporaire ;
- retourne un diagnostic borne ;
- ne lance aucune boucle automatique.

Lors d'une correction, cree un nouveau worktree depuis le nouveau HEAD parent
et rejoue le commit encore en attente. Si le rejeu entre en conflit, nettoie
le nouveau worktree et conserve le changement precedent intact.

Apres un crash ou une interruption, capture les changements si possible,
marque l'execution interrompue puis nettoie le worktree.

Ajoute les metadonnees Git au rapport existant comme donnees factuelles
bornees. Ne change pas le transport, l'ordre ou la logique de reveil des
rapports.

Ne bloque pas la reponse finale du parent lorsqu'une branche temporaire reste
en attente. Le parent conserve strictement la boucle de `main`.

### Permissions enfant

Ajoute un emetteur interne optionnel dedie aux autorisations.

- Utilise l'emetteur courant pour une session normale.
- Utilise l'emetteur du parent pour une session enfant.
- Execute toujours l'outil sous l'identite de l'enfant.
- Enregistre `AllowSession` pour la session enfant.
- Ne change aucun evenement Tauri public ni aucune commande de reponse.

Transmets ce parametre dans les chemins API et Ollama sans modifier leur
ordre, leurs conditions ou leur cadence.

### Verrouillage du mode

Ajoute une famille persistante interne `chat` ou `tools`.

Lors du premier envoi :

- verrouille Chatbot sur la famille `chat` ;
- verrouille Demander l'autorisation ou Acces complet sur la famille `tools` ;
- autorise ensuite uniquement le basculement entre Demander l'autorisation et
  Acces complet pour une session `tools`.

Applique la meme regle aux anciennes sessions lors de leur prochain envoi.
Fais respecter cette regle dans le backend et dans l'interface.

Affiche toujours le vrai libelle choisi. N'affiche jamais `Agent` comme nom de
mode.

### Dossier de session disparu

Verifie le dossier de session juste avant l'envoi, sans modifier la boucle
agentique.

Si le dossier n'existe plus :

- suspends uniquement l'envoi courant ;
- affiche le dossier manquant ;
- trouve le parent existant le plus proche ;
- propose `Switcher` et `Creer` pres du selecteur de projet ;
- mets a jour la session vers le parent avec `Switcher` ;
- recree uniquement le chemin vide avec `Creer` ;
- reprends automatiquement l'envoi apres succes.

Ne restaure aucun fichier. N'affiche aucune erreur technique brute. Ajoute les
textes visibles en francais, anglais, espagnol, allemand, italien, chinois et
japonais.

## Zones protegees

Ne change pas la logique des zones suivantes :

- les boucles agent API et Ollama ;
- `subagent_orchestration.rs` et `subagent_orchestration_context.rs` ;
- `stream_buffer.rs` ;
- l'ajout, le retrait et l'injection des rapports caches ;
- la gestion des messages en file ;
- la cadence des rappels et des controles ;
- la classification `work` et `final`.

Dans les boucles agent, autorise uniquement l'ajout et la transmission du
parametre d'emetteur d'autorisation.

Dans le cycle de fin enfant, autorise uniquement l'appel de capture Git avant
la construction du rapport et l'ajout des metadonnees au rapport. Conserve
l'ordre, les statuts, la file, l'emission des evenements et le nettoyage deja
presents sur `main`.

## Changements interdits

N'ajoute pas :

- une machine d'etat parent/enfant ;
- un nouveau controle initial ou periodique ;
- une nouvelle temporisation ;
- un nouveau verrou de reponse finale ;
- une nouvelle logique de reveil ;
- une nouvelle politique de rapport ;
- un nouveau role de message pour les rapports ;
- un acquittement differe des rapports ;
- une nouvelle gestion des relances ;
- une limite courte ou un resume destructeur des rapports ;
- une boucle de resolution automatique des conflits Git.

Ne corrige pas un probleme theorique hors de ce contrat. Documente-le
separement et demande une decision utilisateur.

## Erreurs de la branche rejetee

La branche `codex/subagent-runtime-hardening` a touche 197 fichiers avec
environ 9 987 ajouts et 1 675 suppressions. Ce volume a masque une reecriture
du flux parent/enfant qui n'etait pas demandee.

Le test manuel de la session
`b76a1056-7f46-4eac-895d-0117ba9f7cbf` a montre :

- 27 appels modele en moins de quatre minutes ;
- 24 appels d'outils avant la vraie attente ;
- des `get_subagent` toutes les deux a quatre secondes ;
- deux `message_subagent` espaces d'environ vingt secondes ;
- des consignes de finalisation ajoutees inutilement aux files enfants ;
- une synthese complete pendant la phase de travail ;
- des rapports recopies et repetes ;
- un etat `running` alors qu'un resume complet etait deja disponible.

Ne reproduis jamais ces causes :

- ne transforme pas une consigne de prompt en boucle runtime ;
- ne laisse pas un controle automatique enchainer des outils sans borne ;
- ne force pas un enfant a finaliser parce qu'il travaille encore ;
- ne confonds pas classification `work` et prevention reelle d'une synthese ;
- ne reveille pas le parent avec un nouveau protocole de rapport ;
- ne change pas plusieurs sous-systemes dans un meme commit ;
- ne considere pas des tests structurels comme preuve du comportement reel du
  LLM.

## Strategie de commits

Utilise cet ordre et ne melange pas les lots :

- `Document minimal subagent hardening scope`
- `Harden subagent tool profiles`
- `Align subagent prompts and inherited context`
- `Add ephemeral coder worktree lifecycle`
- `Route child permissions to parent stream`
- `Lock session modes without changing agent flow`
- `Recover missing session directories`

Avant chaque commit :

- affiche `git diff --stat` ;
- liste les fichiers modifies ;
- verifie qu'ils appartiennent au lot courant ;
- verifie que les zones protegees sont intactes ;
- arrete si le diff devient disproportionne.

## Validation manuelle obligatoire

Avant le premier changement de code, execute sur la branche fraiche un test de
reference avec deux explorateurs. Conserve les logs.

Repete le meme scenario apres chaque lot backend lie aux sous-agents. Verifie :

- le lancement parallele de deux explorateurs ;
- la poursuite du travail utile du parent ;
- l'absence de controle en boucle ;
- l'absence de `message_subagent` automatique ;
- l'absence de demande de rapport anticipee ;
- l'absence de synthese complete pendant la phase de travail ;
- un seul rapport final exploitable par enfant ;
- une reponse finale conforme au comportement de `main`.

Ne declare jamais le flux valide uniquement parce que les tests unitaires
passent.

## Validation automatisee

Ajoute uniquement des tests lies aux changements autorises :

- egalite exacte entre outils decrits, exposes et executables ;
- refus des outils retires avant autorisation ;
- Bash explorateur autorise et interdit ;
- confinement fichiers et Bash du codeur ;
- heritage modele, provider, thinking, langue, personnalite et `AGENTS.md` ;
- cycle Git ajout, modification, suppression, commit, nettoyage, application,
  abandon, conflit et reprise ;
- permission enfant affichee dans le parent et appliquee a l'enfant ;
- verrouillage des familles de mode ;
- actions `Switcher` et `Creer`.

Termine par :

- les tests Rust cibles ;
- `cargo test` ;
- `cargo clippy --all-targets` ;
- `cargo check` ;
- les tests frontend cibles ;
- `npm test` ;
- `npx tsc --noEmit` ;
- la mise a jour Graphify ;
- le controle final du diff par rapport a `main`.

## Porte d'approbation

Apres le commit de ce document, arrete le travail. Demande a l'utilisateur de
le relire et de l'approuver. Ne modifie aucun fichier de code avant cette
approbation explicite.
