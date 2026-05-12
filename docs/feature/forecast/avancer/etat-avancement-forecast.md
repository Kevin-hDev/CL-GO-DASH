# État d'avancement Forecast

Date : 2026-05-12

## Résumé

Le module Forecast est maintenant **utilisable sur ses bases principales**, mais il n'est **pas encore terminé**.

Les fondations critiques sont en place :
- contrat de données Forecast
- graphe principal réel
- installation / suppression de modèles
- sidecar local réel
- Chronos-2 avec contexte et multi-séries
- sélection de modèles
- tools agent Forecast
- premier workflow Scénarios

Il reste encore des blocs fonctionnels importants à finir :
- scénarios contextuels réels
- comparaisons
- analyse avancée
- qualité des données
- exports réels

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
- Sidecar local Chronos réel
- Chronos-Bolt local fonctionnel
- Chronos-2 local fonctionnel
- Chronos-2 avec contexte passé / futur connu
- Chronos-2 multi-séries
- TimeGPT multi-séries câblé côté app

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

## En cours

### Scénarios
- L'UX principale est en place
- Les scénarios simples et contextuels sont câblés
- Le bloc doit encore être validé visuellement dans l'application

## À faire

### Scénarios — version complète
- Valider le flux UI complet sur un fichier avec covariables futures
- Ajuster l'ergonomie si la saisie de plusieurs covariables est trop dense
- Tester TimeGPT contextuel avec une vraie clé API

### Comparaisons
- Comparer prévision principale / scénario
- Comparer plusieurs analyses
- Comparer plusieurs modèles
- Afficher les écarts en valeur et en pourcentage

### Analyse
- Remplir l'onglet Analyse
- Expliquer tendance, risque, points marquants
- Préparer une lecture claire par l'utilisateur et par l'agent

### Qualité des données
- Détecter et afficher données manquantes
- Signaler les lignes ignorées
- Signaler les valeurs corrigées ou fragiles
- Afficher clairement les limites du jeu de données

### Exports
- CSV réel
- JSON réel
- XLSX réel
- PNG / SVG
- PDF rapport

## Points de vigilance

- `TimeGPT` multi-séries est câblé mais doit encore être validé contre une vraie clé / API live
- `cargo check` passe
- Le lint global du repo n'est pas entièrement propre hors scope Forecast
- `graphify-out/` ne doit pas être commit

## Prochain step recommandé

Valider `Scénarios` en conditions réelles dans l'UI, puis seulement attaquer `Comparaisons`.
