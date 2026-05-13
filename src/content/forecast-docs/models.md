# Modèles

Un modèle est le moteur qui calcule la prévision. Forecast peut utiliser des moteurs locaux ou cloud selon le besoin : rapidité, confidentialité, qualité, contexte ou multi-séries.

## Chronos-Bolt

Chronos-Bolt est un moteur local rapide.

Il est adapté quand :

- la prévision doit rester simple ;
- l'utilisateur veut un résultat local ;
- le dataset contient peu ou pas de covariables ;
- le besoin principal est de produire rapidement une courbe future.

Chronos-Bolt est utile pour tester une cible, vérifier un dataset ou obtenir une première projection.

## Chronos-2

Chronos-2 est le moteur local avancé de Forecast.

Il est adapté quand :

- le dataset contient des variables de contexte ;
- le futur connu est disponible ;
- plusieurs séries doivent être traitées ;
- les scénarios contextuels doivent relancer le modèle ;
- l'utilisateur veut garder l'exécution locale.

Chronos-2 est le bon choix quand la prédiction dépend fortement du contexte.

## TimeGPT

TimeGPT est un moteur cloud spécialisé dans les séries temporelles.

Il est adapté quand :

- une clé API Nixtla est configurée ;
- la qualité de prédiction est prioritaire ;
- le dataset est riche ;
- le multi-séries ou les covariables doivent être exploités côté cloud ;
- l'envoi des données au provider est acceptable.

TimeGPT peut être plus puissant, mais il implique une dépendance externe.

## Choisir le bon modèle

| Besoin | Modèle conseillé |
| --- | --- |
| Test rapide local | Chronos-Bolt |
| Prévision locale avec contexte | Chronos-2 |
| Scénarios avec covariables | Chronos-2 ou TimeGPT |
| Multi-séries local | Chronos-2 |
| Qualité cloud avancée | TimeGPT |

Le meilleur modèle dépend surtout du dataset. Un modèle avancé ne compense pas des données mal structurées.
