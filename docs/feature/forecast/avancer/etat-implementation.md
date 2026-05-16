# État réel de l'implémentation Forecast — Mai 2026

Date : 2026-05-16

## Résumé

Le module Forecast est maintenant **fonctionnel sur ses fondations principales**.

On n'est plus du tout dans un état "structure vide".
Le cœur produit existe maintenant :

- import et parsing des données ;
- sélection du modèle ;
- prédiction locale ;
- affichage des résultats ;
- historique des analyses ;
- scénarios ;
- comparaisons ;
- analyse synthétique ;
- support agent ;
- premières couches de contexte et de visualisation.

Forecast n'est **pas terminé**, mais il est **réellement utilisable**.

## État validé par bloc

| Bloc | État | Validation |
| --- | --- | --- |
| Moteur local Forecast | Fonctionnel | Validé |
| Chronos-2 | Fonctionnel avec contexte et multi-séries | Validé côté app |
| TimeGPT | Câblé côté app | À valider avec vraie clé API |
| Tools agent | `forecast`, `forecast_read`, `forecast_analyze` | Validé |
| Vue principale | Graphe, filtres, variables, scénarios, zoom, historique | Validé |
| Scénarios | Ajustement global, covariables, relance modèle, sélection, liste prédictions | Validé V1 |
| Comparaisons | Comparaison base/scénario/prévision compatible, résumé, tableau | Validé V1 |
| Analyse | Tendance, incertitude, points marquants, anomalies, variables | Validé V1 |
| Notes | Notes Markdown locales, timeline, preview, édition, suppression, ouverture éditeur OS | Validé V1 |
| Documentation Forecast | Fenêtre externe Tauri, contenu pédagogique, définitions des variables | Validé fonctionnel |
| Familles de modèles | 11 familles / 25 variantes cataloguées | Dispatch runtime complet côté app |
| Réglages fins modèles | Page `Config` ajoutée | Schémas, édition, sauvegarde, validation et transmission runtime branchés |
| Registry moteurs | Toutes familles déclarées | Adapters locaux branchés, statuts installés/runnable corrigés |
| Réglage matériel global | Hérité dans `Config` | Affiché en lecture seule, application runtime fine à finaliser |
| Backtesting / baselines | Non terminé | À faire |
| Exports | V1 centralisée backend | CSV, XLSX, JSON, PNG, SVG, PDF et clipboard branchés |
| Qualité des données | Non traité | Utile mais non prioritaire |
| Slash commands Forecast | Documenté comme besoin | À implémenter |

Les fichiers `*-lv2.md` décrivent les versions avancées à viser plus tard. Ils ne bloquent pas la validation V1 des blocs déjà terminés.

---

## Ce qui fonctionne déjà

### Données

- Lecture de fichiers CSV / Excel
- Conversion `file_path -> JSON`
- Sauvegarde de l'historique réel dans les analyses
- Reconstruction propre des dates
- Support `series_column` pour le multi-séries
- Colonnes de contexte transmises au backend

### Moteurs Forecast

- Chronos-Bolt local fonctionnel
- Chronos-2 local fonctionnel
- Chronos-2 avec contexte passé / futur connu
- Chronos-2 multi-séries
- TimeGPT multi-séries câblé côté app

### Familles de modèles Forecast

Présent dans le catalogue et visible dans les paramètres Forecast :

| Famille | Variantes cataloguées | État réel |
| --- | --- | --- |
| Chronos-Bolt | Tiny, Mini, Small, Base | Installable et exécutable via sidecar Chronos |
| Chronos-2 | 120M | Installable et exécutable via sidecar Chronos-2 |
| TimesFM 2.5 | 200M | Installable et dispatché vers adapter TimesFM |
| TimeGPT-2.x / 2.1 | Mini, Standard, Pro, 2.1 | API Nixtla branchée, 2.1 inclus côté runtime |
| Datadog Toto 2.0 | 4M, 22M, 313M, 1B, 2.5B | Installable et dispatché vers adapter Toto |
| Salesforce MOIRAI 2.0 | R Small | Installable et dispatché vers adapter MOIRAI |
| IBM FlowState | R1, R1.1 | Installable et dispatché vers adapter FlowState |
| TabPFN-TS | TabPFN-TS, TabPFN-TS-3 | Installable et dispatché vers adapter TabPFN-TS |
| TiRex | 35M | Installable et dispatché vers adapter TiRex |
| Kairos | 10M, 23M, 50M | Installable et dispatché vers adapter Kairos |
| Sundial | 128M | Installable et dispatché vers adapter Sundial |

État réel :

- 11 familles sont visibles dans l'UI Forecast ;
- 25 variantes sont déclarées dans le catalogue backend ;
- le catalogue et les fiches modèles existent ;
- le registry runtime déclare maintenant les 11 familles et 25 variantes ;
- le sidecar local démarre avec `model_id` + `family_id` ;
- les dépendances Python sont installées à la demande par famille ;
- `list_forecast_models` expose `installed`, `installable`, `runnable`, `provider_configured`, `runtime_ready` ;
- Chronos-Bolt / Chronos-2 sont validés fonctionnellement dans l'app ;
- Toto, MOIRAI et Kairos ont été validés sur des prédictions réelles locales ;
- TimesFM, FlowState, TabPFN-TS, TiRex et Sundial ont maintenant un adapter Forecast réel côté sidecar, à valider modèle par modèle après installation ;
- les adapters locaux renvoient un format commun mono-série ou multi-séries (`series_id`) pour éviter les réponses inutilisables côté app ;
- la validation live complète reste à faire pour les familles non encore testées dans l'app ;
- TimeGPT est câblé côté API, 2.1 inclus, mais doit encore être validé avec une vraie clé Nixtla.

### Sidecar local

- Sidecar Forecast réel
- Chargement du vrai moteur local
- Relance du sidecar selon le modèle demandé
- Intégration backend Rust <-> sidecar Python
- Dispatch par famille de modèle
- Installation des dépendances Python par famille
- Stamps par famille pour éviter les réinstallations inutiles
- Erreurs utilisateur génériques si une dépendance ou un adapter échoue

### Vue principale

- Vrai graphe Forecast basé sur Apache ECharts
- Axe temporel continu ECharts pour un déplacement plus fluide
- Historique
- Prévision
- Plage de confiance
- Filtres
- Resize manuel du graphe
- Variables affichables
- Scénarios affichables
- Zoom / déplacement horizontal / reset du graphe via `dataZoom`

### Scénarios

- Ajustement global en `%`
- Scénarios contextuels basés sur covariables
- Relance réelle du modèle pour les scénarios contextuels
- Édition / suppression
- Sélection d'un scénario existant
- Aperçu dans l'onglet `Scénarios`
- Variables visibles dans le graphe scénario
- Navigation directe entre prédictions depuis `Scénarios`

### Comparaisons

- Onglet `Comparaisons` fonctionnel
- Comparaison prévision de base vs scénario
- Comparaison prévision de base vs autre prévision compatible
- Filtrage de compatibilité :
  - même cible
  - même fréquence
  - même horizon
- Support multi-séries
- Graphe comparatif avec zoom / déplacement / reset / resize
- Résumé des écarts :
  - écart moyen
  - écart max
  - écart moyen en pourcentage
  - direction globale
- Tableau période par période avec référence / différence / écart

### Analyse

- Onglet `Analyse` fonctionnel
- Résumé de tendance :
  - direction
  - variation totale
  - valeur de début
  - valeur de fin
- Lecture d'incertitude :
  - plage moyenne
  - plage max
  - période la plus incertaine
- Points marquants :
  - point le plus haut
  - point le plus bas
  - plus forte hausse
  - plus forte baisse
- Détection simple d'anomalies
- Lecture des variables de contexte les plus mouvantes
- Accordéons animés
- Support multi-séries

### Historique

- Liste des analyses Forecast
- Sélection d'une analyse
- Indicateur discret pour les analyses qui ont des scénarios

### Notes

- Notes Markdown locales par analyse
- Timeline dédiée avec points de notes
- Liste chronologique des notes
- Preview Markdown dans l'app
- Création / édition inline / suppression depuis l'UI
- Ouverture du fichier `.md` dans l'éditeur par défaut de l'OS
- Synchronisation des anciennes annotations LLM vers des fichiers Markdown
- Ligne de chemin local séparée du contenu de la note
- Confirmation de suppression alignée sur les clés API

### Agent tools

- `forecast`
- `forecast_read`
- `forecast_analyze`
- annotations
- création / mise à jour / suppression de scénarios

### UI modèles

- Browser modèles par familles
- Sélection du modèle dans le panel Forecast
- Fiches modèles
- Installation / suppression de modèles
- Descriptions et images tirées des sources Hugging Face / GitHub quand disponibles
- Design aligné sur le pattern Ollama pour les fiches modèles

### Config modèles

- Sous-page `Config / Modèles` dans Paramètres > Forecast
- Pattern UI repris d'Ollama :
  - sous-liste latérale ;
  - icône Modelfile pour `Config` ;
  - icône Modèles pour le catalogue ;
  - fiche verrouillée par défaut ;
  - bouton `Éditer`, puis `Annuler / Sauvegarder`
- Liste `Config` limitée aux modèles réellement utilisables :
  - modèles locaux installés ;
  - modèles cloud dont le provider est configuré
- Sauvegarde sparse des paramètres :
  - seule une valeur différente du défaut est persistée ;
  - valeur vide = retour au défaut
- Validation Rust des types, bornes et options
- Transmission des paramètres effectifs au runtime local via `model_config`
- Transmission des paramètres TimeGPT applicables au payload Nixtla, en mono-série et multi-séries
- `horizon_max_override` appliqué côté validation backend
- `context_length` appliqué aux adapters locaux compatibles
- `quantiles` appliqué aux adapters locaux compatibles
- `quantiles` custom remappés proprement vers les bornes affichées quand le moteur ne retourne pas exactement `q10/q90`
- `non_negative_output` appliqué uniquement quand le réglage est activé et que la cible métier est positive
- `output_type` TiRex branché
- `probabilistic_output` TabPFN-TS branché
- `dtype` Sundial branché
- Réglages matériels globaux affichés en héritage :
  - `device`
  - timeout de déchargement modèle
- Sidecar Python adapté pour lire uniquement les paramètres connus par chaque adapter
- Les paramètres non réellement câblés ont été retirés des fiches pour éviter les faux réglages
- Clés i18n ajoutées dans les 7 langues

### i18n / thèmes

- Texte branché sur les 7 langues
- Tokens Forecast dans les deux thèmes
- UI alignée sur le thème existant

### Documentation Forecast

- Bouton Docs visible uniquement quand le panel Forecast est ouvert
- Ouverture dans une fenêtre Tauri externe
- Scroll fonctionnel
- Fermeture indépendante de la fenêtre principale
- Contenu réécrit pour expliquer le forecasting, les scénarios, les datasets et les variables
- Définitions des variables de contexte disponibles dans l'app

---

## Ce qui est encore partiel

### Scénarios

- le cœur marche ;
- mais le bloc demandera encore une passe niveau 2 plus tard ;
- surtout sur duplication, plage de dates et ergonomie avec beaucoup de variables.

### Comparaisons

- la section est utilisable ;
- le niveau 2 est documenté dans `comparaison-lv2.md` ;
- les améliorations restantes concernent surtout multi-sélection, classement, exports et analyse plus avancée.

### Analyse

- la page est maintenant remplie ;
- les calculs sont utiles mais restent simples ;
- une version avancée pourra ajouter vraie décomposition, importance causale des variables et analyse agentique plus poussée.

### TimeGPT

- le multi-séries est câblé côté app ;
- mais la validation réelle avec une vraie clé API reste à faire.

### Familles de modèles

- les 11 familles prévues sont présentes dans le catalogue ;
- le runtime sait maintenant dispatcher toutes les familles cataloguées ;
- les modèles locaux installés sont détectés correctement dans `forecast_models` ;
- le sélecteur Forecast force le modèle choisi dans l'UI quand une prédiction est lancée par l'agent ;
- la validation live reste à compléter pour les familles installables mais pas encore testées dans l'app.

### Réglages fins des modèles

- la page `Config` existe maintenant ;
- les paramètres pertinents par famille sont visibles sans ajout/suppression de lignes ;
- les valeurs sont éditables uniquement en mode édition ;
- les valeurs sauvegardées sont relues au lancement d'une prédiction ;
- les paramètres non applicables à une famille ne sont pas affichés ;
- chaque paramètre visible doit être réellement câblé côté runtime ;
- les réglages `device` et timeout restent hérités des paramètres avancés globaux.

Paramètres maintenant câblés ou exposés selon support réel :

- horizon maximal ;
- quantiles ;
- niveau de confiance TimeGPT ;
- batch size ;
- `dtype` Sundial ;
- longueur d'historique utilisée ;
- gestion des valeurs manquantes ;
- paramètres spécifiques TimeGPT ;
- paramètres spécifiques Toto ;
- paramètres spécifiques MOIRAI ;
- paramètres spécifiques FlowState ;
- paramètres spécifiques TabPFN-TS ;
- paramètres spécifiques TiRex ;
- paramètres spécifiques Kairos ;
- paramètres spécifiques Sundial.

Paramètres communs à prévoir :

| Paramètre | Usage |
| --- | --- |
| Longueur d'historique | Quantité de passé envoyée au modèle |
| Niveau de confiance / quantiles | Largeur et type d'incertitude affichée |
| Batch size | Compromis vitesse / mémoire |
| Device | CPU / GPU selon le réglage matériel global |
| Dtype | Float32 / float16 / bfloat16 / auto selon les moteurs |
| Gestion des valeurs manquantes | Nettoyer ou conserver les trous selon le modèle |
| Sortie positive | Empêcher les bornes négatives sur les cibles métier positives |

Paramètres spécifiques exposés par famille :

| Famille | Paramètres spécifiques utiles |
| --- | --- |
| Chronos-Bolt | horizon max, longueur d'historique, quantiles, sortie positive |
| Chronos-2 | horizon max, longueur d'historique, quantiles, sortie positive |
| TimesFM 2.5 | horizon max, longueur d'historique, quantiles, sortie positive |
| TimeGPT-2.x / 2.1 | horizon max, `level`, `clean_ex_first`, `finetune_steps`, `finetune_loss`, `finetune_depth`, `feature_contributions` |
| Toto 2.0 | horizon max, longueur d'historique, `decode_block_size`, `has_missing_values`, quantiles, sortie positive |
| MOIRAI 2.0 | horizon max, longueur d'historique, batch size, quantiles |
| FlowState | horizon max, `scale_factor`, `batch_first`, quantiles, sortie positive |
| TabPFN-TS | horizon max, sortie probabiliste, longueur d'historique, quantiles, sortie positive |
| TiRex | horizon max, type de sortie, quantiles, sortie positive |
| Kairos | horizon max, `preserve_positivity`, `average_with_flipped_input`, `generation`, quantiles, sortie positive |
| Sundial | horizon max, nombre d'échantillons, longueur d'historique, dtype, quantiles, sortie positive |

Paramètres à ne pas exposer pour l'instant :

- `temperature`, `top_k`, `top_p` pour Chronos-Bolt / Chronos-2 : pas pertinent pour l'usage Bolt/2 actuel ;
- paramètres d'architecture internes : couches, dimensions, patch length, decoder type ;
- paramètres d'entraînement : learning rate, epochs, LoRA, fine-tuning local ;
- options SDK non utilisées par CL-GO-DASH ;
- réglages réseau internes comme timeout/retry, sauf écran développeur plus tard.

### Réglage matériel

- Forecast doit réutiliser le réglage CPU/GPU global déjà présent dans `/paramètres/avancé` ;
- il ne faut pas créer un deuxième réglage matériel isolé dans Forecast ;
- le sidecar Forecast devra respecter ce réglage comme Ollama.

### Socle graphique

- le moteur graphique est maintenant Apache ECharts ;
- l'intégration directe `echarts/core` est conservée, sans wrapper `echarts-for-react` ;
- l'axe X est migré en axe temps continu, plus adapté au zoom et au déplacement ;
- le zoom et le déplacement horizontal utilisent `dataZoom`.

---

## Ce qui reste à faire

### Familles de modèles Forecast

- garder le catalogue actuel des 11 familles ;
- compléter la validation live sur toutes les familles installables ;
- vérifier chaque adapter avec :
  - mono-série ;
  - multi-séries si supporté ;
  - covariables si supportées ;
  - quantiles custom ;
  - sortie positive ;
  - horizon limité par config.

### Réglages fins modèles

- continuer les tests réels par famille ;
- vérifier les paramètres qui peuvent modifier fortement le résultat ;
- appliquer plus finement le réglage matériel global au sidecar Forecast ;
- documenter les paramètres dans la documentation utilisateur Forecast.

### Registry moteurs

- centraliser les moteurs Forecast disponibles ;
- déclarer leurs capacités :
  - contexte ;
  - futur connu ;
  - multi-séries ;
  - quantiles ;
  - backtesting ;
  - fine-tuning ;
  - anomalies ;
  - imputation ;
- utiliser ce registry dans l'UI, les tools agent et le backend.

État actuel :

- catalogue et runtime sont branchés ;
- le sélecteur Forecast force le modèle choisi ;
- les statuts `installed`, `runnable` et `runtime_ready` ont été corrigés pour les modèles locaux testés ;
- il reste à enrichir les capacités fines par modèle et à les utiliser partout.

### Réglage matériel Forecast

- analyser le flux actuel Ollama ;
- réutiliser le réglage global CPU/GPU ;
- appliquer ce réglage au sidecar Forecast ;
- relancer proprement le moteur si nécessaire.

### Backtesting / baselines

- comparer une prédiction avec l'historique réel disponible ;
- ajouter des baselines simples ;
- préparer AutoGluon / StatsForecast / NeuralForecast comme moteurs d'évaluation ou comparaison.

### Scénarios

- amélioration niveau 2 documentée dans `scenario-lv2.md`
- affichage encore plus clair des variables modifiées
- duplication de scénario
- ciblage par plage de dates
- raffinements UX si beaucoup de variables
- validation `TimeGPT` contextuel

### Analyse

- amélioration niveau 2 documentée dans `analyse-lv2.md`
- vraie décomposition tendance / saisonnalité / résidu
- importance des variables plus fiable
- résumé automatique généré par l'agent

### Qualité des données

- analyse informative, non bloquante ;
- signaler les données manquantes ;
- signaler les lignes ignorées ;
- signaler les valeurs suspectes ;
- signaler les limites du dataset ;
- permettre à l'utilisateur de continuer quand même dès que le modèle peut techniquement recevoir une entrée.

### Notes

- la V1 est validée ;
- le niveau 2 est documenté dans `notes-lv2.md` ;
- les améliorations restantes concernent surtout les liens avancés avec scénarios, variables, sources web et pièces jointes.

### Exports

La V1 est branchée avec une logique centralisée côté backend.

Formats disponibles :

- CSV
- JSON
- XLSX
- PNG
- SVG
- PDF rapport
- Copier clipboard

Contenu exporté :

- métadonnées de l'analyse ;
- historique ;
- prévisions ;
- quantiles et plage de confiance quand disponibles ;
- scénarios ;
- annotations ;
- notes Markdown locales ;
- données d'entrée sauvegardées dans l'analyse.

Comportement :

- commande backend unique `export_forecast_analysis` ;
- fichiers générés dans le dossier Téléchargements de l'OS ;
- nommage automatique à partir du nom de l'analyse et de son identifiant ;
- export clipboard sans fichier intermédiaire ;
- PNG / SVG générés depuis les données sauvegardées de l'analyse.

Améliorations possibles plus tard :

- mise en page PDF plus avancée ;
- rendu image calé exactement sur le graphe ECharts affiché à l'écran ;
- presets d'exports selon usage agent, rapport ou partage.

### Slash commands Forecast

- créer une couche d'aide rapide pour les LLM et les utilisateurs
- `/forecast`
- `/forecast-predict`
- `/forecast-dataset`
- `/forecast-cmd`
- puis `/forecast-scenarios`, `/forecast-view`, `/forecast-models`

### Finition UI

- une grosse passe de polish Forecast a été faite et validée ;
- il restera une passe finale globale après `Qualité des données` et la validation des exports en usage réel.

---

## Points de vigilance

- `graphify-out/` ne doit pas être commit
- le lint global du repo n'est pas totalement propre hors Forecast
- `cargo check` passe
- `npx tsc --noEmit` passe sur le chantier Forecast
- plusieurs fichiers docs du repo principal sont anciens ou décalés par rapport à la branche Forecast active

---

## Position réelle du chantier

Forecast n'est plus une promesse ou une maquette.

On a maintenant :

- une base technique sérieuse ;
- un vrai moteur local ;
- une vraie UI Forecast ;
- des scénarios qui fonctionnent ;
- un graphe exploitable ;
- un usage agentique réel.

La suite logique est maintenant :

1. valider les exports Forecast en usage réel sur plusieurs analyses
2. compléter la validation live des modèles non encore testés
3. compléter le registry moteurs et les capacités réelles fines
4. brancher plus finement Forecast sur le réglage matériel global
5. ajouter les slash commands Forecast
6. ajouter ensuite la qualité des données comme lecture informative
7. refaire une passe finale documentation + UI + i18n
8. lancer une validation end-to-end complète avec un agent LLM
