# État d'avancement Forecast

Date : 2026-05-16

## Résumé

Le module Forecast est maintenant **utilisable sur ses blocs principaux**, mais il n'est **pas encore terminé**.

Les fondations critiques sont en place :
- contrat de données Forecast
- graphe principal réel
- installation / suppression de modèles
- sidecar local réel
- modèles locaux multi-familles
- sélection de modèles
- sélecteur Forecast forcé
- configuration modèle par famille
- exports centralisés backend
- tools agent Forecast
- scénarios, comparaisons, analyse et notes

Il reste encore des blocs fonctionnels importants à finir :
- validation live de toutes les familles de modèles
- registry de capacités plus fin
- slash commands Forecast
- qualité des données

## Fait

### Phase 1 — Contrat de données
- Historique réel sauvegardé dans les analyses
- Dates réelles reconstruites proprement
- Colonnes de contexte transmises au backend
- Gestion `file_path` CSV / Excel fiable
- Support multi-séries avec `series_column`

### Phase 2 — Vue principale
- Vrai graphe `ECharts`
- Historique + prévision + plage de confiance
- Filtres Forecast
- Resize manuel du graphe
- Lecture plus explicite des périodes et des valeurs

### Modèles et moteurs
- Browser modèles par familles
- Sélection du modèle dans le panel Forecast
- Sélecteur Forecast persistant
- Modèle sélectionné forcé pour les tools agent Forecast
- Sidecar local Chronos réel
- Chronos-Bolt local fonctionnel
- Chronos-2 local fonctionnel
- Chronos-2 avec contexte passé / futur connu
- Chronos-2 multi-séries
- TimeGPT multi-séries câblé côté app
- 11 familles / 25 variantes présentes dans le catalogue Forecast
- Runtime par famille branché côté backend / sidecar
- Dépendances Python installées à la demande par famille
- Toto, MOIRAI et Kairos validés sur prédictions locales
- TimesFM, FlowState, TabPFN-TS, TiRex et Sundial à valider en conditions réelles après installation

### Tools agent
- `forecast` propre
- `forecast_read` propre
- `forecast_analyze` propre
- Support agent pour annotations
- Support agent pour création / édition / suppression de scénarios

### Scénarios
- Création de scénarios simples par ajustement global
- Édition / suppression
- Aperçu du scénario dans l'onglet
- Courbe scénario visible dans la vue principale
- Indicateur discret sur les prédictions avec scénarios
- Sélection directe d'une prédiction depuis la page Scénarios
- Mini panneau latéral groupé par mois
- Scénarios contextuels basés sur modification de covariables futures
- Relance réelle du modèle pour les scénarios contextuels

### Comparaisons
- Comparaison prévision / scénario
- Comparaison entre analyses compatibles
- Résumé des écarts
- Tableau période par période
- Support multi-séries
- Graphe comparatif avec zoom / déplacement / reset

### Analyse
- Tendance
- Incertitude
- Points marquants
- Anomalies simples
- Variables de contexte les plus mouvantes
- Accordéons animés

### Notes
- Notes Markdown locales par analyse
- Timeline de notes
- Preview Markdown
- Création / édition / suppression
- Ouverture dans l'éditeur OS

### Config modèles
- Sous-page `Config / Modèles` dans Paramètres > Forecast
- Paramètres visibles uniquement pour les modèles utilisables
- Édition verrouillée par défaut
- Sauvegarde sparse : valeur vide = retour au défaut
- Validation Rust des types, bornes et options
- Transmission des réglages au runtime local et TimeGPT
- Paramètres non câblés retirés pour éviter les faux réglages

### Exports
- Commande backend centralisée `export_forecast_analysis`
- CSV réel
- JSON réel
- XLSX réel multi-feuilles
- PNG réel
- SVG réel
- PDF rapport
- Copier clipboard
- Export de l'analyse complète : métadonnées, historique, prévisions, quantiles, scénarios, annotations, notes et données d'entrée sauvegardées
- Fichiers générés dans le dossier Téléchargements de l'OS

## En cours

### Modèles
- Compléter la validation live sur toutes les familles installables
- Vérifier les capacités réelles par famille : contexte, quantiles, covariables, multi-séries
- Enrichir le registry de capacités fines

## À faire

### Slash commands Forecast
- `/forecast`
- `/forecast-predict`
- `/forecast-dataset`
- `/forecast-cmd`
- `/forecast-scenarios`
- `/forecast-models`

### Qualité des données
- Détecter et afficher données manquantes
- Signaler les lignes ignorées
- Signaler les valeurs corrigées ou fragiles
- Afficher clairement les limites du jeu de données

## Points de vigilance

- `TimeGPT` multi-séries est câblé mais doit encore être validé contre une vraie clé / API live
- Toutes les familles sont branchées côté runtime, mais toutes ne sont pas encore validées visuellement dans l'app
- `cargo check` passe
- Le lint global du repo n'est pas entièrement propre hors scope Forecast
- `graphify-out/` ne doit pas être commit

## Prochain step recommandé

Valider les exports Forecast en usage réel, puis passer aux slash commands Forecast ou à la qualité des données.
