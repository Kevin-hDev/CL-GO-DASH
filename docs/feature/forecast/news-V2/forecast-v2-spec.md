# Forecast V2 — Spécification produit, UX et architecture

**Statut :** décisions produit validées
**Date :** 21 juillet 2026
**Périmètre :** Forecast dans les sessions Agent Local
**Document de référence :** cette spec prévaut sur les anciens documents Forecast lorsqu'ils la contredisent.

## 1. Résumé des décisions validées

Forecast reste directement lié au LLM et à la session de conversation.

Tu appliques les décisions suivantes :

1. Tu conserves le chat comme centre de commande de Forecast.
2. Tu conserves le panneau latéral comme surface principale de lecture des résultats.
3. Tu ne transformes pas Forecast en onglet principal indépendant.
4. Tu ne remplaces jamais le chat par un espace Forecast.
5. Tu ne récupères jamais l'espace de la conversation pour afficher une interface Forecast complète.
6. Tu retires à terme le mode plein écran qui agrandit le panneau au détriment du chat.
7. Tu proposes une fenêtre Forecast secondaire pour les tâches manuelles complexes.
8. Tu lies cette fenêtre à la session et à l'analyse Forecast actives.
9. Tu n'ajoutes aucun bouton « demander une explication » : l'utilisateur écrit simplement sa demande dans le chat.
10. Tu proposes deux modes de sélection du modèle : **Manuel** et **Auto**.
11. Tu affiches exactement le libellé **Auto**, sans « par l'agent » ni autre précision dans le libellé principal.
12. Tu fais filtrer les contraintes techniques et matérielles par le backend.
13. Tu fais choisir le modèle final par le LLM uniquement parmi les candidats compatibles.
14. Tu n'ajoutes le contexte matériel au LLM que pendant une opération Forecast qui en a besoin.
15. Tu donnes la priorité à la validation des données, aux backtests, aux baselines et à la calibration avant d'ajouter aveuglément de nouveaux modèles.
16. Tu reprends le design, les composants, les polices et les tokens de CL-GO-DASH.
17. Tu ne hardcodes aucune couleur, taille de layout réutilisable, durée ou texte visible.
18. Tu gardes chaque écran lisible et tu évites d'entasser trop de contrôles différents dans une même zone.

## 2. Vision produit

Le parcours principal reste agentique :

```text
L'utilisateur décrit son besoin dans le chat
  → le LLM recherche ou prépare les données
  → le LLM contrôle la qualité des données
  → le LLM choisit ou respecte le modèle selon la politique active
  → le moteur Forecast produit et sauvegarde la prévision
  → le panneau affiche le résultat
  → l'utilisateur poursuit la discussion dans le même chat
```

L'utilisateur peut ensuite demander naturellement au LLM :

- d'interpréter une évolution ;
- d'expliquer une anomalie ou une incertitude ;
- de relancer une prévision ;
- de comparer plusieurs modèles ;
- de créer ou modifier un scénario ;
- de rechercher des facteurs externes ;
- de produire un rapport ou un export.

Le panneau n'est pas un éditeur complet. Il affiche en priorité un résultat compréhensible, vérifiable et manipulable rapidement.

La fenêtre Forecast secondaire apporte un espace manuel plus riche sans séparer Forecast de la conversation.

## 3. Principes de conception

### 3.1 Le chat pilote

Tu gardes le LLM comme orchestrateur principal. Tu ne dupliques pas le chat dans la fenêtre Forecast et tu ne crées pas de conversation secondaire.

### 3.2 Le panneau permet de lire

Tu optimises le panneau pour les actions courtes : lire, filtrer, activer une couche, changer d'analyse, exporter et ouvrir la fenêtre avancée.

### 3.3 La fenêtre permet de travailler

Tu places dans la fenêtre secondaire les actions qui demandent plusieurs étapes, de grands tableaux, plusieurs colonnes ou une configuration détaillée.

### 3.4 Le backend reste la source de vérité

Tu ne synchronises pas deux copies indépendantes d'une analyse. Tu fais charger et modifier le même état backend par le panneau, la fenêtre et les tools LLM.

### 3.5 La qualité prime sur l'apparence de précision

Tu ne présentes jamais un modèle comme « meilleur » sans mesure sur les données concernées. Avant les backtests, tu parles de modèle « compatible » ou « recommandé selon ses capacités ».

## 4. Objectifs de la V2

Tu atteins les objectifs suivants :

- tu gardes Forecast naturel dans une session Agent Local ;
- tu rends le panneau durable malgré l'ajout de fonctions avancées ;
- tu fournis une vraie interface manuelle sans masquer la conversation ;
- tu rends la sélection automatique du modèle sûre et explicable ;
- tu prends en compte les ressources disponibles au moment du lancement ;
- tu mesures réellement la qualité des prévisions ;
- tu rends chaque analyse reproductible et auditable ;
- tu rends les adaptateurs modèles conformes aux capacités réellement supportées ;
- tu gardes l'interface claire en panneau étroit comme dans une grande fenêtre ;
- tu conserves une architecture testable et maintenable.

## 5. Hors périmètre

Tu n'introduis pas les comportements suivants :

- un onglet Forecast principal indépendant d'Agent Local ;
- un chat dédié dans la fenêtre Forecast ;
- une WebView Forecast qui possède sa propre copie des analyses ;
- un bouton qui envoie automatiquement une demande d'explication au LLM ;
- une sélection de modèle cloud sans autorisation préalable de l'utilisateur ;
- une promesse de « meilleur modèle » fondée uniquement sur une description marketing ;
- un prompt global contenant en permanence le matériel de la machine ;
- des formulaires complexes entassés dans le panneau latéral ;
- des couleurs ou dimensions CSS hardcodées dans les composants.

## 6. Architecture des surfaces UI

### 6.1 Vue d'ensemble

```text
Session Agent Local
├── Conversation avec le LLM
│   └── reste toujours disponible dans la fenêtre principale
├── Panneau Forecast
│   └── consultation et interactions rapides
└── Fenêtre Forecast
    └── préparation, configuration, évaluation et édition avancées
```

Le panneau et la fenêtre utilisent les mêmes analyses, la même politique de modèle et les mêmes événements backend.

### 6.2 Contrat du panneau Forecast

Tu conserves dans le panneau :

- le graphique principal ;
- l'historique et la prévision ;
- les intervalles d'incertitude ;
- les KPI réellement calculés ;
- le statut de qualité des données ;
- les avertissements importants ;
- le modèle effectivement utilisé ;
- la source du choix du modèle, Manuel ou Auto ;
- les filtres de couches ;
- l'activation rapide de scénarios existants ;
- un résumé des comparaisons ;
- un résumé de l'analyse ;
- les notes récentes en lecture ;
- l'historique des analyses ;
- les exports rapides ;
- le sélecteur de modèle ;
- le bouton d'ouverture de la fenêtre Forecast.

Tu retires progressivement du panneau :

- l'import et la correction détaillée des données ;
- les grands formulaires de configuration ;
- les réglages avancés des modèles ;
- la création complexe de scénarios ;
- les tableaux de backtest détaillés ;
- les comparaisons multi-modèles complètes ;
- les tableaux de données modifiables ;
- l'édition longue de notes ou de rapports.

Tu appliques la règle suivante :

> Si l'utilisateur lit ou active rapidement un résultat, garde l'action dans le panneau. Si l'utilisateur prépare, configure, compare ou modifie plusieurs éléments, ouvre la fenêtre Forecast.

### 6.3 Comportement du panneau

Tu conserves son ouverture automatique lorsqu'une analyse est créée par le LLM.

Tu conserves son lien avec la session active.

Tu conserves son redimensionnement horizontal pour la lecture.

Tu remplaces le contrôle de plein écran Forecast par un contrôle d'ouverture de la fenêtre Forecast. Tu ne fais plus grandir Forecast en supprimant visuellement le chat.

Tu gardes les vues invisibles non interactives. Tu utilises `inert` ou un démontage conditionnel en plus de `aria-hidden` afin que les contrôles hors écran ne restent pas accessibles au clavier.

Tu ne montes pas les écrans lourds tant qu'ils ne sont pas nécessaires.

### 6.4 Contenu du panneau selon sa largeur

Tu bases les adaptations sur la largeur réelle du panneau avec des container queries. Tu n'utilises pas la largeur globale de la fenêtre comme seule référence.

Tu définis trois densités :

| État | Intention | Présentation |
| --- | --- | --- |
| Étroit | Lire l'essentiel | une colonne, graphique simplifié, contrôles prioritaires |
| Moyen | Lire et filtrer | une ou deux colonnes, filtres complets, résumés |
| Large | Explorer le résultat | graphique enrichi, KPI et tableaux de lecture |

Tu centralises les seuils dans des tokens ou constantes de layout. Tu ne dupliques pas les valeurs dans plusieurs CSS.

Tu empêches tout composant enfant d'imposer une largeur supérieure au panneau.

## 7. Fenêtre Forecast secondaire

### 7.1 Rôle

Tu crées une fenêtre Tauri secondaire, décorée, redimensionnable et non modale. Tu la considères comme une extension de la session, pas comme une nouvelle section autonome de l'application.

Tu ne la rends pas toujours au premier plan. L'utilisateur doit pouvoir revenir librement au chat principal.

Tu utilises un titre contextuel de type `Forecast — <nom de l'analyse>`.

### 7.2 Ouverture

Tu places un bouton d'ouverture dans le header du panneau Forecast.

Tu utilises une icône existante et un tooltip i18n. Tu affiches un libellé complet lorsque l'espace le permet et uniquement l'icône lorsque le panneau est étroit.

Tu réutilises le pattern `WebviewWindow` déjà présent pour la documentation Forecast.

Tu utilises une route dédiée, par exemple `/#/forecast-workbench`, et tu ajoutes une branche d'entrée explicite dans `App.tsx`.

### 7.3 Instance et bornage

Tu gardes une seule fenêtre Forecast avancée par instance de l'application. Tu évites de créer une WebView par analyse ou par session.

Quand la fenêtre existe déjà :

1. tu sauvegardes ou persistes le brouillon courant ;
2. tu changes son contexte vers la session et l'analyse demandées ;
3. tu la remontres si nécessaire ;
4. tu lui donnes le focus.

Tu bloques le changement de contexte si la persistance du brouillon échoue. Tu affiches uniquement une erreur utilisateur générique et localisée.

### 7.4 Contexte de session

Tu lies la fenêtre aux deux identifiants suivants :

- `session_id` ;
- `analysis_id`, facultatif tant qu'aucune analyse n'existe.

Tu valides strictement ces identifiants dans le backend avant toute lecture ou mutation.

Tu n'inclus pas de dataset brut dans les événements de synchronisation.

Quand l'analyse active change dans le panneau Forecast, tu synchronises immédiatement
le contexte partagé si la fenêtre est ouverte. Tu appliques directement dans la fenêtre
le snapshot de contexte validé reçu par `forecast-workbench-context-changed`, sans attendre
une seconde lecture. Ce snapshot contient uniquement les identifiants, noms, section active
et numéros de révision nécessaires au changement de contexte, jamais le dataset ni l'analyse
complète. Si plusieurs sélections arrivent rapidement, tu garantis que la dernière sélection
devient l'état final.

Dans le header de la fenêtre, tu affiches uniquement le nom de l'analyse et le sélecteur
de modèle. Tu n'affiches ni libellé de session active, ni nom de session, car ces
informations sont redondantes avec le contexte Forecast déjà visible.

### 7.5 Fermeture et restauration

Tu n'arrêtes jamais la session quand la fenêtre Forecast se ferme.

Tu ne supprimes jamais un brouillon à la fermeture sans sauvegarde explicite.

Tu mémorises la taille et la position de la fenêtre dans les données de l'application. Tu vérifies au prochain lancement que la fenêtre reste visible sur un moniteur disponible.

Tu centralises les dimensions par défaut et minimales dans un fichier de constantes. Tu ne disperses pas ces dimensions dans les composants.

### 7.6 Permissions Tauri

Tu crées une capability dédiée, par exemple `forecast-workbench.json`.

Tu lui accordes uniquement les permissions nécessaires à la fenêtre et aux événements. Tu ne lui accordes pas un accès direct et général au système de fichiers.

Tu fais passer les lectures et écritures Forecast par des commandes backend validées.

### 7.7 Synchronisation

Tu fais du backend la source de vérité.

Tu utilises des événements compacts, par exemple :

- `forecast-workbench-context-changed` ;
- `forecast-analysis-updated` ;
- `forecast-selection-policy-changed` ;
- `forecast-models-changed` ;
- `forecast-run-status-changed`.

Les événements d'analyse restent compacts : tu inclus uniquement des identifiants, versions
et statuts, puis tu fais relire l'analyse complète par une commande dédiée. Seul l'événement
de contexte de la fenêtre transporte son petit snapshot validé défini en 7.4.

Tu ajoutes une version ou un numéro de révision à chaque analyse afin de détecter une modification concurrente du panneau, de la fenêtre ou du LLM.

Tu refuses une écriture basée sur une ancienne révision et tu demandes un rechargement. Tu ne laisses pas deux modifications s'écraser silencieusement.

## 8. Organisation de la fenêtre Forecast

### 8.1 Direction visuelle

Tu adoptes une direction **analytique, calme, précise et utilitaire**.

Tu reprends l'identité de CL-GO-DASH : surfaces sobres, hiérarchie discrète, accent orange existant, police Geist Sans, valeurs techniques en JetBrains Mono et animations courtes.

Tu ne crées pas une esthétique de dashboard générique avec une accumulation de cartes identiques.

Tu structures la fenêtre en deux surfaces distinctes :

- une sidebar vitrée sur toute la hauteur, qui regroupe le nom de l'analyse,
  le sélecteur de modèle et la navigation Forecast ;
- un espace de travail sombre, opaque, arrondi et séparé de la sidebar par
  une marge constante.

Tu n'ajoutes aucun header transversal au-dessus de ces deux surfaces. Sur une
largeur réduite, la sidebar devient une bande supérieure compacte et l'espace
de travail conserve sa propre surface et son propre défilement.

Tu privilégies :

- une hiérarchie claire ;
- de l'espace entre les groupes ;
- une action principale par zone ;
- des détails révélés progressivement ;
- des tableaux seulement lorsqu'ils sont utiles ;
- des résumés avant les réglages avancés.

### 8.2 Navigation

Tu regroupes les fonctions par intention plutôt que d'afficher une longue liste plate.

Structure cible :

| Espace | Contenu principal |
| --- | --- |
| Données | import, aperçu, mapping, qualité, corrections |
| Prévision | cible, horizon, fréquence, modèle, paramètres, lancement |
| Évaluation | baselines, backtests, métriques, calibration |
| Comparaison | classement, différences, compromis qualité/vitesse |
| Scénarios | création, modification, duplication, plages de dates |
| Rapport | notes longues, provenance, exports avancés |

Tu n'affiches pas simultanément tous les réglages de ces espaces.

Tu affiches les paramètres avancés dans un panneau secondaire interne ou un accordion explicite fermé par défaut.

Tu gardes le contexte de l'analyse visible pendant la navigation.

### 8.3 Répartition panneau / fenêtre

| Fonction | Panneau | Fenêtre |
| --- | --- | --- |
| Lire le graphique principal | complet | complet |
| Activer une couche existante | oui | oui |
| Lire les KPI | oui | oui |
| Lire les avertissements | oui | oui |
| Importer et corriger les données | accès vers fenêtre | complet |
| Configurer une prévision | résumé | complet |
| Choisir Manuel ou Auto | oui | oui, même état |
| Modifier les paramètres du modèle | accès vers fenêtre | complet |
| Lire une comparaison | résumé | complet |
| Lancer un backtest | accès vers fenêtre ou LLM | complet |
| Afficher un scénario | oui | oui |
| Créer un scénario complexe | accès vers fenêtre ou LLM | complet |
| Lire des notes | résumé | complet |
| Éditer un rapport | non | complet |
| Export rapide | oui | oui |
| Export avancé | accès vers fenêtre | complet |

## 9. Design system et CSS

### 9.1 Tokens

Tu réutilises en priorité les tokens présents dans :

- `src/styles/tokens.css` pour le layout, les espacements, les rayons, les tailles et la typographie ;
- `src/styles/themes/dark.css` pour les couleurs sombres ;
- `src/styles/themes/light.css` pour les couleurs claires.

Si un besoin n'est couvert par aucun token :

1. tu vérifies qu'un token équivalent n'existe pas ;
2. tu ajoutes un token sémantique ;
3. tu l'ajoutes dans les deux thèmes s'il concerne une couleur ;
4. tu l'ajoutes dans `tokens.css` s'il concerne le layout ;
5. tu réutilises ce token partout où le besoin est identique.

Tu ne hardcodes aucune couleur dans un composant Forecast.

Tu ne hardcodes une valeur inline que si elle est calculée dynamiquement au runtime.

### 9.2 Classes

Tu gardes les styles colocalisés avec les composants.

Tu réutilises les classes réellement partagées existantes.

Tu préfixes les nouvelles classes de la fenêtre Forecast avec un préfixe dédié, par exemple `fcw-`.

Tu vérifies avant création que le préfixe et les classes ne sont pas déjà utilisés.

Tu ne redéfinis pas une classe appartenant à un autre composant.

### 9.3 Thèmes et OS

Tu testes chaque écran en thème sombre et clair.

Tu utilises une surface de fenêtre stable et opaque. Tu évites d'empiler plusieurs couches translucides ou plusieurs `backdrop-filter`.

Tu respectes les classes OS centralisées. Tu ne recrées pas une détection macOS, Linux ou Windows dans Forecast.

### 9.4 Lisibilité

Tu limites la densité visuelle :

- tu ne places pas plusieurs formulaires complexes dans une seule carte ;
- tu n'affiches pas plus d'une action principale par bloc ;
- tu regroupes les champs qui répondent à la même question ;
- tu gardes les descriptions courtes à proximité des décisions importantes ;
- tu utilises les tooltips pour les détails secondaires ;
- tu évites les rangées de boutons uniquement composées d'icônes ambiguës ;
- tu conserves une largeur lisible pour les libellés et les valeurs.

Tu affiches les séries temporelles continues avec un lissage monotone sur
l'axe du temps. La courbe passe par chaque valeur réelle, ne relie pas les
données manquantes et ne crée pas de pic entre deux points. Tu appliques le
même lissage aux bandes d'incertitude, aux scénarios et aux variables. Une
série réellement discrète ou en paliers peut explicitement conserver un tracé
droit.

### 9.5 Responsive

Tu utilises des container queries pour le panneau et la fenêtre.

Tu passes les grilles complexes en une colonne avant qu'elles ne deviennent illisibles.

Tu limites les tableaux par un scroll local explicite sans faire défiler toute la fenêtre horizontalement.

Tu gardes les actions essentielles visibles sans chevauchement pendant un redimensionnement.

Tu testes au minimum :

- panneau à 250 px ;
- panneau à 320 px ;
- panneau à 360 px ;
- panneau à 600 px ;
- fenêtre à sa largeur minimale ;
- fenêtre maximisée ;
- thèmes sombre et clair ;
- macOS, Windows et Linux.

### 9.6 Accessibilité

Tu rends toutes les fonctions utilisables au clavier.

Tu maintiens un focus visible avec les tokens existants.

Tu fournis des noms accessibles aux boutons icônes.

Tu utilises les rôles adaptés pour les onglets, menus, listes, tableaux et dialogues.

Tu respectes `prefers-reduced-motion` pour les animations non essentielles.

Tu n'utilises jamais la couleur comme seul indicateur d'un statut.

Tu maintiens un ordre de focus cohérent après un changement de section ou de contexte.

## 10. i18n

Tu ajoutes tous les textes visibles dans les sept langues :

- français ;
- anglais ;
- espagnol ;
- allemand ;
- italien ;
- chinois ;
- japonais.

Tu n'écris aucun texte visible en dur dans React ou Rust.

Tu utilises exactement les libellés français suivants dans le sélecteur :

- `Manuel` ;
- `Auto`.

Tu gardes les explications complémentaires dans une description ou un tooltip, sans allonger le libellé `Auto`.

Tu ajoutes des tests qui vérifient la présence des nouvelles clés dans les sept fichiers.

## 11. Sélection du modèle

### 11.1 Politique persistée

Tu remplaces le simple modèle sélectionné par une politique structurée :

```rust
ForecastSelectionPolicy {
    mode: Manual | Auto,
    manual_model_id: Option<String>,
    allow_cloud_in_auto: bool,
}
```

Tu gardes cette préférence au niveau de l'application, comme le modèle sélectionné actuel.

Tu enregistres la politique dans le data dir centralisé avec une écriture atomique.

Tu conserves séparément le dernier modèle manuel afin qu'il soit restauré immédiatement lorsque l'utilisateur quitte Auto.

Tu utilises Manuel comme valeur par défaut après migration afin de ne pas changer silencieusement le comportement existant.

### 11.2 Présentation dans le sélecteur

Tu places le contrôle de mode directement sous le champ de recherche et avant la liste des modèles.

```text
Rechercher un modèle…

Sélection
[ Manuel ] [ Auto ]

────────────────────
Modèles disponibles
```

Tu gardes ce contrôle visible même lorsqu'une recherche filtre la liste.

En mode Manuel :

- tu affiches le nom du modèle sélectionné dans le trigger ;
- tu marques le modèle actif dans la liste ;
- tu forces ce modèle lors des appels LLM et UI.

En mode Auto :

- tu affiches `Auto` dans le trigger avant toute nouvelle exécution ;
- affiche `Auto · Chronos-2` après une exécution lorsque l'indication du modèle effectivement utilisé est utile ;
- tu ne modifies pas le dernier modèle manuel ;
- tu fais repasser en Manuel lorsqu'un utilisateur clique directement sur un modèle.

Tu fais utiliser la même politique par le sélecteur compact, la fenêtre Forecast et la configuration de lancement.

### 11.3 Mode Manuel

Tu conserves le comportement de contrôle utilisateur :

- le backend impose le modèle manuel ;
- le LLM ne modifie pas la sélection persistée ;
- le tool expose clairement `selection_policy: manual` ;
- le tool `forecast` n'accepte pas silencieusement un autre modèle ;
- le backend retourne une erreur structurée si un modèle différent est demandé.

### 11.4 Mode Auto

Tu interprètes Auto comme une obligation de sélectionner un modèle lorsque l'utilisateur n'en impose pas un dans son message.

Tu n'interprètes pas Auto comme une simple permission facultative.

Tu appliques l'ordre de priorité suivant :

1. Tu respectes un modèle explicitement demandé par l'utilisateur pour cette exécution, s'il est compatible et autorisé.
2. Sinon, tu filtres les modèles techniquement admissibles dans le backend.
3. Tu fournis au LLM une liste courte et classée de candidats.
4. Tu fais choisir un candidat au LLM selon le besoin utilisateur.
5. Tu valides une seconde fois les ressources juste avant de charger le modèle.
6. Tu lances la prévision avec le modèle effectif.
7. Tu sauvegardes le modèle, la source du choix et les raisons principales.

Tu ne modifies pas le modèle manuel persistant après un choix Auto.

### 11.5 Cloud

Tu limites Auto aux modèles locaux par défaut.

Tu places l'autorisation des modèles cloud dans les paramètres Forecast afin de ne pas surcharger le sélecteur.

Tu n'inclus un modèle cloud que si :

- l'utilisateur a activé `allow_cloud_in_auto` ;
- le provider est configuré ;
- le modèle est exécutable ;
- la politique de données autorise l'envoi externe.

Tu ne fais jamais basculer silencieusement du local vers le cloud lorsque les ressources sont insuffisantes.

## 12. Tool de sélection et pilotage du LLM

### 12.1 Décision de nommage

Tu gardes le tool existant `forecast_models` pour éviter un tool de mutation supplémentaire.

Tu fais évoluer sa responsabilité : il ne se contente plus de lister le catalogue ; il retourne la politique active et, en mode Auto, les candidats adaptés à la tâche.

Le sélecteur UI active une politique Manuel ou Auto. Il n'active pas directement un tool visible par l'utilisateur.

### 12.2 Comportement dynamique

En mode Manuel, `forecast_models` retourne :

```json
{
  "selection_policy": {
    "mode": "manual",
    "forced_model": "chronos-2"
  }
}
```

En mode Auto, `forecast_models` retourne une réponse compacte :

```json
{
  "selection_policy": {
    "mode": "auto",
    "cloud_allowed": false
  },
  "task_profile": {
    "series_kind": "multi_series",
    "horizon": 30,
    "frequency": "D",
    "past_covariates": true,
    "future_covariates": true,
    "probabilistic_required": true
  },
  "candidates": [
    {
      "model_id": "chronos-2",
      "compatibility": "recommended",
      "resource_fit": "comfortable",
      "reasons": ["multi_series", "future_covariates", "horizon_supported"]
    }
  ],
  "selection_basis": "capabilities_and_resources"
}
```

Tu bornes `candidates` à cinq éléments.

Tu bornes les raisons par candidat et tu utilises des codes stables plutôt que de longues explications.

Tu gardes les exclusions détaillées hors du contexte principal. Tu ne retournes qu'un résumé borné des exclusions lorsque le LLM en a besoin.

### 12.3 Profil de tâche

Tu fais considérer au minimum :

- nombre de points historiques ;
- nombre de séries ;
- horizon ;
- fréquence ;
- présence de valeurs manquantes ;
- besoin de variables passées ;
- besoin de variables futures ;
- besoin d'intervalles probabilistes ;
- cible positive ou non ;
- préférence de l'utilisateur entre précision, vitesse, local et cloud.

Dans une première étape, tu acceptes un profil compact fourni par le pipeline Forecast et tu le revalides dans le backend.

Lorsque l'audit de données V2 est disponible, tu fais référencer le profil validé par un identifiant borné plutôt que de renvoyer les données brutes entre plusieurs tools.

### 12.4 Instructions données au LLM

En mode Auto, tu ajoutes uniquement dans la description dynamique du tool Forecast les obligations suivantes :

- tu appelles `forecast_models` avant une première prévision ou après un changement matériel du besoin ;
- tu choisis uniquement un modèle présent dans `candidates` ;
- tu privilégies les scores de backtest lorsqu'ils existent ;
- tu respectes une demande explicite de l'utilisateur si elle passe les contrôles ;
- tu ne qualifies pas un modèle de meilleur si aucun backtest comparable n'existe ;
- tu relances la sélection si le backend signale que les ressources ont changé ;
- tu ne modifies jamais la politique persistée de l'utilisateur.

Tu n'ajoutes aucune de ces informations au prompt global lorsque Forecast n'est pas actif.

## 13. Ressources matérielles

### 13.1 Répartition des responsabilités

Tu fais mesurer et appliquer les contraintes par le backend. Tu ne demandes pas au LLM de calculer la mémoire nécessaire.

Tu utilises les informations existantes du catalogue :

- RAM estimée ;
- VRAM estimée ;
- support CPU ;
- support GPU ;
- taille sur disque ;
- capacités ;
- horizon maximal.

Tu utilises les détecteurs existants de mémoire totale et utilisée.

### 13.2 Instant de mesure

Tu mesures les ressources :

1. au moment de construire les candidats Auto ;
2. juste avant de charger le modèle choisi.

Tu prends en compte les ressources déjà utilisées par le LLM local, Ollama et les autres moteurs actifs.

Tu n'utilises pas uniquement la mémoire totale de la machine.

### 13.3 Mémoire unifiée et dédiée

Tu distingues :

- la VRAM dédiée sur les GPU compatibles ;
- la mémoire unifiée sur Apple Silicon ;
- la RAM disponible pour une exécution CPU ;
- les cas où la mesure est inconnue.

Tu conserves une marge de sécurité pour le système et le LLM actif. Tu centralises cette marge dans une configuration ou des constantes documentées et testées.

Tu ne sélectionnes pas un gros modèle lorsque la détection est inconnue. Tu limites alors la sélection à un profil léger sûr ou tu retournes qu'aucun candidat local n'est garanti.

### 13.4 Informations envoyées au LLM

Tu ne fournis pas au LLM le nom complet de la machine, un numéro de série, la liste des périphériques ou d'autres informations inutiles.

Tu retournes seulement des catégories utiles :

- `comfortable` ;
- `acceptable` ;
- `tight` ;
- `cpu_only` ;
- `cloud` ;
- `unknown`.

Tu conserves les chiffres détaillés dans le backend et dans les diagnostics locaux non sensibles.

## 14. Algorithme de sélection Auto

### 14.1 Filtres bloquants

Tu exclus un modèle si une condition suivante échoue :

- modèle absent du catalogue ;
- adapter de prédiction absent ;
- modèle local non installé ;
- runtime local indisponible ;
- provider cloud non configuré ;
- cloud non autorisé ;
- fréquence non supportée ;
- horizon supérieur à la limite effective ;
- séries multiples non supportées ;
- variables nécessaires non supportées ;
- sortie probabiliste nécessaire mais indisponible ;
- CPU ou GPU incompatible ;
- mémoire disponible insuffisante avec marge de sécurité.

Tu revalides ces conditions au lancement. Tu bloques en cas de changement défavorable et tu ne substitues pas silencieusement un autre modèle.

### 14.2 Classement avant backtest

Tant qu'aucun backtest n'existe, tu classes les candidats selon :

- adéquation aux caractéristiques des données ;
- marge matérielle ;
- stabilité connue de l'adapter ;
- vitesse attendue ;
- préférence local/cloud ;
- besoin de variables externes ;
- besoin d'une prévision probabiliste.

Tu étiquettes ce classement `capabilities_and_resources`.

### 14.3 Classement avec backtest

Lorsque les backtests sont disponibles, tu donnes la priorité à :

- MASE ou RMSSE ;
- sMAPE ;
- MAE ;
- perte quantile ou CRPS ;
- biais ;
- couverture réelle des intervalles ;
- stabilité entre les fenêtres de validation ;
- temps d'exécution ;
- mémoire maximale observée.

Tu compares toujours les modèles à une baseline simple.

Tu n'utilises pas MAPE comme métrique unique, notamment lorsque les valeurs peuvent être nulles ou proches de zéro.

Tu étiquettes ce classement `rolling_backtest`.

### 14.4 Choix final du LLM

Tu fais utiliser au LLM le besoin exprimé par l'utilisateur pour départager les candidats sûrs :

- précision maximale ;
- rapidité ;
- fonctionnement local ;
- coût cloud ;
- besoin d'explication ;
- besoin de variables externes ;
- besoin de plusieurs séries.

Tu sauvegardes une raison courte et structurée. Tu ne demandes pas au LLM de produire une justification longue à chaque exécution.

## 15. Fondations de fiabilité Forecast

### 15.1 Audit des données

Avant une prévision V2, tu contrôles :

- validité des dates ;
- ordre chronologique ;
- doublons ;
- périodes manquantes ;
- cohérence avec la fréquence ;
- quantité minimale d'historique ;
- longueur d'historique par rapport à l'horizon ;
- valeurs numériques invalides ;
- valeurs manquantes ;
- valeurs extrêmes ;
- changements de régime ;
- identifiants de séries ;
- disponibilité réelle des variables futures ;
- risque de fuite d'une information future.

Tu n'inventes jamais une date silencieusement lorsqu'une date fournie est invalide.

Tu n'utilises jamais la dernière ligne comme date maximale sans vérifier l'ordre chronologique.

Tu traites correctement les jours ouvrés et tu ne les assimiles pas à des jours calendaires.

Tu bloques les erreurs structurelles qui rendent la prévision invalide. Tu présentes les avertissements non bloquants lorsque le modèle peut encore travailler correctement.

### 15.2 Import CSV et tableur

Tu conserves la position originale des colonnes même lorsqu'un header est vide.

Tu refuses ou renomme explicitement les headers dupliqués. Tu ne laisses jamais une colonne écraser silencieusement une autre.

Tu valides les séparateurs décimaux sans remplacer naïvement chaque virgule.

Tu limites la taille d'entrée, le nombre de lignes, le nombre de colonnes et la longueur de chaque valeur.

### 15.3 Backtests

Tu implémentes une validation temporelle glissante.

Tu gardes strictement le futur en dehors des données d'entraînement de chaque fenêtre.

Tu adaptes le nombre de fenêtres à la longueur de la série et à l'horizon.

Tu exposes un avertissement lorsque l'historique est trop court pour une évaluation fiable.

### 15.4 Baselines

Tu ajoutes au minimum :

- Naive ;
- Seasonal Naive ;
- Drift ;
- ETS ou Theta lorsque la série le permet.

Reporte ARIMA et les moteurs AutoML à une étape ultérieure si les baselines minimales couvrent déjà la première livraison.

Tu signales clairement lorsqu'un modèle avancé ne bat pas la baseline.

### 15.5 Intervalles et calibration

Tu appliques réellement le niveau de confiance choisi.

Tu valides l'ordre et la cardinalité des quantiles reçus.

Tu ne remplaces jamais une valeur invalide ou un intervalle manquant par zéro.

Tu calibres les intervalles avec des résidus de backtest ou une méthode conforme lorsque les données le permettent.

Tu affiches la couverture mesurée à côté du niveau théorique.

### 15.6 Validation des sorties

Tu vérifies :

- le nombre attendu de prédictions ;
- le nombre attendu de séries ;
- les identifiants de séries ;
- les dates futures ;
- l'ordre des points ;
- les valeurs finies ;
- l'alignement des quantiles ;
- l'horizon effectif ;
- la conformité aux contraintes de positivité réellement activées.

Tu bloques toute réponse partielle ou incohérente. Tu ne la sauvegardes pas comme une analyse valide.

### 15.7 Bornes

Tu limites :

- le nombre de séries ;
- l'horizon ;
- le produit `séries × horizon` ;
- le nombre total de prédictions ;
- la taille des quantiles ;
- la taille des résultats de tools ;
- le nombre de candidats modèles ;
- le nombre de backtests simultanés ;
- le nombre d'analyses chargées en parallèle.

Tu centralises ces limites et tu les testes aux frontières.

## 16. Modèles et adaptateurs

### 16.1 Stratégie

Tu conserves le catalogue moderne existant, mais tu ne considères un modèle comme disponible en Auto que si son adapter a été validé en conditions réelles.

Tu donnes la priorité aux améliorations suivantes :

1. Tu fais de Chronos-2 la première référence locale multivariée après validation complète.
2. Tu complètes le support XReg de TimesFM 2.5 avant d'ajouter un autre modèle similaire.
3. Tu alignes les capacités Toto sur le support officiel réellement disponible.
4. Tu corriges les adapters qui remplacent les dates réelles par des dates artificielles.
5. Tu distingues les modèles réellement multivariés des modèles appliqués indépendamment à chaque série.
6. Tu complètes les variables externes et les intervalles TimeGPT.
7. Tu évalues IBM Granite TTM-R2 comme option locale légère.
8. Tu ajoutes des baselines et des ensembles avant de multiplier les foundation models.

### 16.2 Contrat d'un adapter

Tu exiges pour chaque famille :

- un test mono-série ;
- un test multi-séries lorsque déclaré ;
- un test de variables passées lorsque déclaré ;
- un test de variables futures lorsque déclaré ;
- un test des quantiles ;
- un test de l'horizon maximal ;
- un test des dates et fréquences ;
- un test de valeurs manquantes ;
- un test de sortie positive lorsqu'elle est activée ;
- un test d'échec fermé lorsque la réponse est invalide.

Tu ne déclares une capability `true` qu'après validation de bout en bout dans CL-GO-DASH.

### 16.3 Reproductibilité du runtime

Tu figes les versions de dépendances Python et les révisions des modèles.

Tu n'installes pas automatiquement une branche Git mutable comme unique référence de production.

Tu ajoutes un contrôle d'intégrité ou une révision vérifiable aux téléchargements.

Tu marques un modèle installé uniquement après un smoke test de chargement réussi.

Tu places les modèles nécessitant `trust_remote_code` derrière une politique explicite et documentée.

## 17. Tools Forecast V2

Tu fais évoluer progressivement la surface de tools :

| Tool | Rôle |
| --- | --- |
| `forecast` | lance une prévision validée |
| `forecast_models` | expose la politique et les candidats compatibles |
| `forecast_read` | lit une analyse ou une liste bornée |
| `forecast_analyze` | gère annotations et scénarios autorisés |
| `forecast_data_audit` | contrôle et résume la qualité des données |
| `forecast_backtest` | exécute une évaluation glissante bornée |
| `forecast_compare_models` | compare des résultats homogènes |

Tu n'ajoutes un tool que lorsque son contrat, ses limites et ses erreurs sont testés.

Tu gardes les résultats compacts. Tu sauvegardes les sorties complètes dans le stockage Forecast et tu retournes des identifiants au LLM.

Tu ne retournes pas automatiquement des analyses complètes volumineuses dans le contexte.

Tu conserves les tools Forecast optionnels lorsqu'ils ne sont pas activés pour la session.

## 18. Analyse, anomalies et importance des variables

Tu ne présentes plus une amplitude globale comme une importance réelle de variable.

Tu ne présentes plus un simple z-score sur l'ensemble historique + prévision comme une détection d'anomalie avancée.

Tu utilises :

- des résidus de modèle ou de baseline pour les anomalies ;
- des seuils adaptés à la saisonnalité ;
- une décomposition tendance/saisonnalité/résidu ;
- une permutation, une ablation ou une contribution supportée par le modèle pour l'importance des variables.

Si seule une heuristique simple est disponible, tu la nommes explicitement « signal indicatif ».

## 19. Stockage et provenance

### 19.1 Analyse sauvegardée

Tu ajoutes aux analyses V2 :

- version du schéma ;
- révision de l'analyse ;
- empreinte des données d'entrée ;
- profil de qualité des données ;
- modèle exact ;
- révision du modèle ;
- versions des dépendances principales ;
- configuration effective ;
- source de sélection `manual`, `auto` ou `explicit_user_override` ;
- raisons structurées du choix ;
- classe matérielle utilisée, sans détail sensible ;
- scores de backtest ;
- baseline ;
- calibration et couverture ;
- durée d'exécution ;
- statut complet ou échoué.

Tu rends les nouveaux champs rétrocompatibles avec les analyses V1 via des valeurs par défaut explicites.

### 19.2 Index et rétention

Tu gardes les index légers. Tu ne relis pas des centaines d'analyses complètes pour afficher une liste.

Tu supprimes réellement les anciens fichiers lorsqu'une politique de rétention les évince.

Tu définis une limite de stockage configurable ou une politique de rétention documentée.

Tu empêches les fichiers orphelins après un échec d'indexation.

Tu utilises des écritures atomiques et une protection contre les modifications concurrentes.

### 19.3 Notes et chemins

Tu valides tous les identifiants avant de construire un chemin.

Tu canonicalises et vérifies la racine avant chaque accès dérivé d'une entrée frontend ou LLM.

Tu sauvegardes une note et son index comme une opération cohérente. Tu ne laisses pas un fichier ou un index partiellement mis à jour.

## 20. Performance

Tu appliques les règles suivantes :

- tu ne charges pas toutes les analyses complètes dans une liste ;
- tu pagines les historiques et comparaisons ;
- tu limites les lectures parallèles ;
- tu ne rends pas les vues lourdes invisibles en permanence ;
- tu limites les recalculs ECharts pendant le drag avec `requestAnimationFrame` ou un throttling ;
- tu ne reconstruis pas toutes les options du graphique si seule sa taille change ;
- tu mets les inférences locales dans une file bornée ;
- tu empêches plusieurs gros modèles Forecast de saturer simultanément la mémoire ;
- tu annules proprement les tâches liées à une fenêtre ou une analyse fermée ;
- tu libères les modèles selon la politique `keep_alive` ;
- tu journalises uniquement des métadonnées filtrées et jamais les données brutes ou secrets.

## 21. Sécurité et confidentialité

Tu appliques les règles suivantes :

- tu valides toutes les entrées de tools et commandes Tauri ;
- tu bornes toutes les collections alimentées par l'extérieur ;
- tu gardes les clés API uniquement dans le backend et le vault ;
- tu zéroïses les secrets après usage ;
- tu compares les tokens d'authentification en temps constant ;
- tu ne transmets jamais une clé API à la fenêtre Forecast ;
- tu ne journalises jamais un dataset brut ou un payload provider complet ;
- tu utilises des erreurs utilisateur génériques et i18n ;
- tu n'affiches ni chemins internes, ni stack traces, ni versions de bibliothèques dans l'UI ;
- tu revalides le modèle et les ressources au moment de l'exécution ;
- tu refuses l'envoi cloud lorsque la politique ne l'autorise pas ;
- tu n'accordes à la fenêtre secondaire que les permissions Tauri minimales.

## 22. Architecture de code cible

### 22.1 Frontend

Tu organises les composants par responsabilité :

```text
src/components/forecast/
├── panel/                 # lecture compacte dans la session
├── workbench/             # fenêtre manuelle avancée
├── model-selection/       # Manuel, Auto, candidats et statuts
├── data-quality/          # audit et corrections
├── evaluation/            # backtests, baselines, métriques
├── scenarios/             # composants partagés de scénarios
├── charts/                # visualisations partagées
├── shared/                # composants Forecast réellement communs
└── types/                 # types de présentation bornés
```

Tu n'effectues pas un déplacement mécanique massif si un découpage progressif réduit le risque.

Tu gardes les hooks d'accès aux données séparés des composants visuels.

Tu gardes les actions Tauri dans des services ou hooks dédiés.

Tu évites qu'un composant source ou test dépasse 200 lignes. Tu découpes avant d'atteindre cette limite.

### 22.2 Backend

Tu ajoutes des modules à responsabilité unique, par exemple :

```text
services/forecast/
├── selection_policy.rs
├── model_candidates.rs
├── hardware_profile.rs
├── data_quality/
├── evaluation/
├── provenance.rs
├── workbench_context.rs
└── analysis_revision.rs
```

Tu réutilises `crate::services::paths::data_dir()` pour chaque stockage.

Tu gardes les commandes Tauri fines et tu places la logique métier dans les services.

Tu gardes les adapters Python séparés par famille et tu ajoutes des tests dédiés.

### 22.3 État partagé

Tu évites de faire de `localStorage` la source de vérité des analyses ou de la politique de modèle.

Tu réserves `localStorage` aux états purement visuels non critiques, par exemple une section repliée.

Tu persistes la politique de modèle, les brouillons importants et les révisions dans le backend.

## 23. Migration depuis la V1

Tu effectues une migration non destructive :

1. Tu lis `forecast-selected-model.json` s'il existe.
2. Tu crées la nouvelle politique avec `mode = Manual`.
3. Tu places l'ancien modèle dans `manual_model_id` après validation.
4. Tu désactives `allow_cloud_in_auto` par défaut.
5. Tu écris le nouveau fichier atomiquement.
6. Tu conserves l'ancien fichier jusqu'à validation réussie de la migration ou tu le renomme en sauvegarde bornée.

Tu ajoutes des valeurs `serde(default)` pour les nouveaux champs d'analyse.

Tu continues à afficher les analyses V1. Tu marques simplement les informations de provenance inconnues comme indisponibles.

Tu ne convertis pas les anciennes heuristiques en métriques certifiées.

## 24. Plan d'implémentation

### Phase 1 — Structure UI et politique

Tu réalises :

- la politique Manuel/Auto ;
- la migration du modèle sélectionné ;
- le contrôle sous la recherche ;
- la fenêtre Forecast vide mais synchronisée ;
- la capability Tauri minimale ;
- le remplacement du plein écran par l'ouverture de la fenêtre ;
- la séparation initiale entre composants de lecture et composants manuels ;
- les tests de synchronisation et de migration.

### Phase 2 — Qualité des données et contrats

Tu réalises :

- l'audit de dates, fréquence, doublons et trous ;
- la correction du mapping tableur ;
- les limites globales de sortie ;
- la validation stricte des réponses modèles ;
- l'unification des limites d'horizon ;
- l'application réelle du niveau de confiance ;
- les profils de données réutilisables.

### Phase 3 — Backtests et baselines

Tu réalises :

- la validation temporelle glissante ;
- Naive, Seasonal Naive, Drift et ETS/Theta ;
- les métriques comparables ;
- la calibration des intervalles ;
- les vues Évaluation et Comparaison de la fenêtre ;
- les tools `forecast_backtest` et `forecast_compare_models`.

### Phase 4 — Auto guidé par les données

Tu réalises :

- le profil matériel dynamique ;
- le filtrage de candidats ;
- le classement par compatibilité ;
- le classement par backtest ;
- le choix final du LLM ;
- la revalidation avant chargement ;
- la provenance complète de la sélection.

Si tu livres un Auto initial basé sur les capacités et ressources avant les backtests, étiquette-le honnêtement et ne le présente pas comme une sélection du meilleur modèle.

### Phase 5 — Adaptateurs et fonctions avancées

Tu réalises :

- les corrections TimesFM, Toto, TimeGPT et adapters multivariés ;
- les tests Python de bout en bout ;
- la décomposition réelle ;
- les anomalies sur résidus ;
- l'importance fiable des variables ;
- les ensembles de modèles ;
- les scénarios avancés ;
- les rapports avancés ;
- la surveillance de dérive.

## 25. Stratégie de tests

### 25.1 Rust

Tu testes :

- lecture, validation, migration et écriture atomique de la politique ;
- transitions Manuel vers Auto et Auto vers Manuel ;
- conservation du dernier modèle manuel ;
- cloud désactivé par défaut ;
- filtrage par capability ;
- filtrage par horizon et fréquence ;
- filtrage RAM/VRAM aux valeurs limites ;
- détection matérielle inconnue ;
- revalidation avant chargement ;
- limites de candidats ;
- révisions concurrentes ;
- validation des identifiants de fenêtre, session et analyse ;
- rétention et nettoyage du stockage.

### 25.2 Python

Tu ajoutes des fixtures locales déterministes et tu testes chaque adapter sans téléchargement réseau pendant les tests unitaires.

Tu testes séparément le chargement réel des familles dans une suite d'intégration opt-in.

Tu vérifies le nombre de prédictions, les séries, les dates, les quantiles et les erreurs invalides.

### 25.3 React

Tu testes :

- présence de Manuel et Auto sous la recherche ;
- Auto non affecté par le filtre de recherche ;
- clic sur un modèle qui repasse en Manuel ;
- restauration du dernier modèle manuel ;
- affichage `Auto · modèle` après exécution ;
- bouton d'ouverture de la fenêtre ;
- synchronisation d'analyse ;
- erreurs génériques ;
- focus clavier ;
- vues hors écran inertes ;
- layout aux largeurs de panneau définies ;
- layout à la taille minimale de la fenêtre ;
- thèmes sombre et clair ;
- sept langues.

### 25.4 End-to-end

Tu testes les parcours suivants :

1. LLM lance une prévision en Manuel.
2. LLM lance une prévision en Auto local.
3. LLM respecte un modèle explicitement demandé en Auto.
4. Auto refuse un modèle trop lourd.
5. Auto ne choisit pas le cloud sans autorisation.
6. La fenêtre s'ouvre sur la bonne session et la bonne analyse.
7. Une modification dans la fenêtre met à jour le panneau.
8. Une modification LLM met à jour la fenêtre.
9. La fermeture de la fenêtre ne ferme pas la session.
10. Un conflit de révision ne détruit aucune modification.
11. Une prévision invalide n'est pas sauvegardée.
12. Un modèle avancé est comparé à une baseline.

Tu ne valides jamais une phase si ses tests unitaires, intégration et contrôles statiques applicables ne passent pas.

## 26. Critères d'acceptation produit

Tu considères la V2 conforme lorsque toutes les affirmations suivantes sont vraies :

- le chat reste visible et utilisable pendant l'affichage du panneau Forecast ;
- aucun mode Forecast ne remplace la conversation dans la fenêtre principale ;
- le panneau se concentre sur la lecture des résultats ;
- les workflows manuels complexes s'ouvrent dans une fenêtre secondaire liée à la session ;
- la fenêtre ne contient aucun chat indépendant ;
- fermer la fenêtre ne ferme ni la session ni l'analyse ;
- le panneau ne contient aucun bouton d'explication LLM ;
- le sélecteur affiche exactement Manuel et Auto ;
- Auto se trouve sous la recherche ;
- Auto oblige une sélection lorsque l'utilisateur n'impose pas de modèle ;
- le backend exclut les modèles techniquement ou matériellement incompatibles ;
- le LLM ne reçoit pas le profil matériel en dehors d'une opération Forecast ;
- le LLM choisit uniquement parmi une liste courte de candidats validés ;
- Auto reste local par défaut ;
- les choix Auto n'écrasent pas le dernier modèle Manuel ;
- chaque analyse enregistre le modèle effectif et la source du choix ;
- les dates et sorties invalides échouent sans fabriquer de zéros ou de dates ;
- chaque modèle avancé est comparé à une baseline ;
- les intervalles affichent une couverture mesurée lorsqu'elle est disponible ;
- les adaptations UI utilisent la largeur du conteneur ;
- tous les textes existent dans les sept langues ;
- toutes les couleurs utilisent des tokens présents dans les deux thèmes ;
- aucun composant Forecast ne devient un fichier monolithique ;
- les suites TypeScript, lint, Rust, Python et end-to-end applicables passent.

## 27. Documents liés

Tu utilises les documents existants comme historique et recherche complémentaire :

- `../01-research-context.md` ;
- `../02-models-comparison.md` ;
- `../03-capabilities.md` ;
- `../04-market-gaps.md` ;
- `../05-ux-patterns.md` ;
- `../06-architecture-tools.md` ;
- `../07-architecture-panels.md` ;
- `../08-architecture-providers.md` ;
- `../09-architecture-sidecar-storage.md` ;
- `../10-design-decisions.md` ;
- `../11-design-wireframes.md` ;
- `../12-design-model-management.md` ;
- `../avancer/etat-implementation.md`.

En cas de contradiction, tu appliques cette spec V2, notamment pour :

- la séparation panneau de lecture / fenêtre de travail ;
- la suppression du plein écran qui prend la place du chat ;
- les modes Manuel et Auto ;
- la sélection matérielle contextuelle ;
- les exigences de backtest et de fiabilité ;
- la lisibilité et les container queries.
