# 📊 Simulation de Collecte de Ressources

## Table des Matières
- [Objectif](#objectif)
- [Exigences](#exigences)
  - [Génération de Carte](#génération-de-carte)
- [Types de Robots](#types-de-robots)
  - [Robots Éclaireurs](#robots-éclaireurs)
  - [Robots Collecteurs](#robots-collecteurs)
- [Système de Base](#système-de-base)
- [Architecture Concurrente et Gestion des Connaissances](#architecture-concurrente-et-gestion-des-connaissances)
- [Exigences Techniques](#exigences-techniques)
- [Disposition Visuelle](#disposition-visuelle)
- [Critères de Réussite](#critères-de-réussite)
- [Barème d'Évaluation](#barème-dévaluation)

## Objectif
Créer une simulation graphique en terminal utilisant Ratatui qui simule des robots autonomes collectant des ressources sur une carte générée procéduralement.

## Exigences

### Génération de Carte
- Générer une carte avec des obstacles basés sur du bruit.
- Peupler la carte avec deux types de ressources :
  - Sources d'énergie (représentées par `E`)
  - Gisements de cristaux (représentés par `C`)
- Les ressources doivent avoir des quantités aléatoires (50-200 unités chacune).

## Types de Robots

### Robots Éclaireurs
- Représentés par `x`
- Explorer la carte de manière aléatoire.
- Découvrir et partager les emplacements de ressources.
- Éviter les obstacles connus.
- Ne peuvent pas collecter de ressources.

### Robots Collecteurs
- Représentés par `o`
- Naviguer vers les emplacements de ressources connus.
- Collecter les ressources une unité à la fois.
- Retourner à la base en portant des ressources.
- Décharger les ressources à la base centrale.

## Système de Base
- La base centrale agit comme :
  - Point de départ pour tous les robots.
  - Centre de stockage de ressources et de connaissances.
  - Centre de communication pour partager les découvertes.
- Suivre le total d'énergie et de cristaux collectés.

## Architecture Concurrente et Gestion des Connaissances
- Chaque robot opère comme une entité indépendante avec des connaissances locales limitées.
- Les robots commencent sans information sur la carte au-delà de leur environnement immédiat.
- Le partage d'informations se fait par des mécanismes de communication asynchrone.
  
### Comportements distribués clés à implémenter :
- Les éclaireurs diffusent les ressources et obstacles découverts aux autres robots.
- Les collecteurs communiquent les événements de collecte pour que la base mette à jour l'état global.
- Le système de base coordonne l'agrégation des connaissances de toutes les découvertes robotiques.
- Les robots doivent synchroniser leurs actions sans bloquer les opérations des autres.

## Exigences Techniques
- Utiliser Ratatui pour le rendu de l'interface utilisateur terminal.
- Implémenter une simulation en temps réel.
- Gérer les entrées utilisateur (toute pression de touche quitte).
- Utiliser les fonctionnalités de concurrence de Rust pour la coordination des robots.
- Générer les obstacles en utilisant le bruit de Perlin.

## Disposition Visuelle
- **Obstacles** : `O` (cyan clair)
- **Énergie** : `E` (vert)
- **Cristaux** : `C` (magenta clair)
- **Base** : `#` (vert clair)
- **Éclaireurs** : `x` (rouge)
- **Collecteurs** : `o` (magenta)
  
### UI
Afficher le compteur de ressources collectées.

## Critères de Réussite
- Les robots naviguent de manière autonome et évitent les obstacles.
- Les éclaireurs découvrent et partagent les emplacements de ressources.
- Les collecteurs rassemblent efficacement les ressources et retournent à la base.
- Mises à jour en temps réel du progrès de collecte des ressources.
- Rendu terminal propre avec codage couleur approprié.

## Barème d'Évaluation

### Implémentation de Base (60 points)
- **Génération de Carte (10 points)** : Génération d'obstacles basée sur le bruit, placement des ressources.
  
- **Comportements des Robots (20 points)** : Comportements distincts d'éclaireur et collecteur, pathfinding.
  
- **Système de Base (10 points)** : Stockage des ressources, fonctionnalité de point de départ.
  
- **Système de Communication (20 points)** : Passage de messages, partage de connaissances, synchronisation.

### Qualité Technique (25 points)
- **Architecture Concurrente (10 points)** : Entités robotiques indépendantes, opérations non-bloquantes.
  
- **Intégration Ratatui (8 points)** : Rendu en temps réel, codage couleur approprié.
  
- **Qualité du Code (7 points)** : Structure propre, gestion d'erreurs appropriée, documentation.

### Fonctionnalités Avancées (15 points)
- **Optimisation (5 points)** : Pathfinding efficace, stratégies d'allocation des ressources.
  
- **Robustesse (5 points)** : Gérer les cas limites, épuisement des ressources, évitement de collisions.
  
- **Expérience Utilisateur (5 points)** : Simulation fluide, retour visuel clair.
