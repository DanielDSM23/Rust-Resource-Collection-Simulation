Cahier des Charges : Simulation de Collecte de Ressources
Objectif

Créer une simulation graphique en terminal utilisant Ratatui qui simule des robots autonomes collectant des ressources sur une carte générée procéduralement.
1. Exigences Fondamentales

Génération de la Carte

    Générer une carte avec des obstacles basés sur du bruit (ex: bruit de Perlin).

    Peupler la carte avec deux types de ressources : Sources d'énergie et Gisements de cristaux.

    Attribuer des quantités aléatoires pour chaque ressource (entre 50 et 200 unités chacune).

Système de Base Centrale

    Agir comme point de départ pour tous les robots.

    Servir de centre de stockage de ressources et de connaissances.

    Fonctionner comme centre de communication pour partager les découvertes.

    Suivre et afficher le total d'énergie et de cristaux collectés.

2. Entités et Comportements

Robots Éclaireurs

    Rôle : Explorer la carte de manière aléatoire.

    Actions : Découvrir et partager les emplacements de ressources avec la base et les autres robots.

    Contraintes : Éviter les obstacles connus et ne collecter aucune ressource.

Robots Collecteurs

    Rôle : Rassembler les ressources.

    Actions : Naviguer vers les emplacements de ressources connus, collecter les ressources une unité à la fois, et retourner à la base pour les décharger.

3. Architecture Concurrente et Technique

Architecture Distribuée

    Chaque robot opère comme une entité indépendante avec des connaissances locales limitées.

    Les robots commencent sans information sur la carte au-delà de leur environnement immédiat.

    La synchronisation des actions doit se faire sans bloquer les opérations des autres entités (opérations non-bloquantes).

Système de Communication

    Le partage d'informations s'effectue par des mécanismes de communication asynchrone.

    Les éclaireurs diffusent les ressources et obstacles découverts aux autres robots.

    Les collecteurs communiquent les événements de collecte pour que la base mette à jour l'état global.

    Le système de base coordonne l'agrégation des connaissances de toutes les découvertes robotiques.

Contraintes Techniques

    Utiliser les fonctionnalités de concurrence de Rust pour la coordination des robots.

    Utiliser Ratatui pour le rendu de l'interface utilisateur terminal en temps réel.

    Gérer les entrées utilisateur (quitter l'application sur n'importe quelle pression de touche).

4. Disposition Visuelle et Interface

L'interface doit afficher un rendu terminal propre avec un compteur des ressources collectées en temps réel.
Élément	Caractère	Couleur Ratatui
Obstacles	O	Cyan clair
Énergie	E	Vert
Cristaux	C	Magenta clair
Base	#	Vert clair
Éclaireurs	x	Rouge
Collecteurs	o	Magenta
5. Critères de Réussite

    Navigation autonome des robots et évitement réussi des obstacles.

    Découverte et partage effectifs des emplacements de ressources par les éclaireurs.

    Collecte efficace et retour à la base par les collecteurs.

    Mises à jour fluides et en temps réel du progrès de collecte.

    Interface terminale propre respectant le codage couleur demandé.

6. Barème d'Évaluation (100 points au total)
Catégorie	Points	Détails des attentes
Implémentation de Base	60	
- Génération de Carte	10	Génération d'obstacles basée sur le bruit, placement des ressources.
- Comportements des Robots	20	Comportements distincts (éclaireur/collecteur), algorithme de pathfinding.
- Système de Base	10	Stockage des ressources, fonctionnalité de point de départ.
- Système de Communication	20	Passage de messages, partage de connaissances, synchronisation.
Qualité Technique	25	
- Architecture Concurrente	10	Entités robotiques indépendantes, opérations non-bloquantes.
- Intégration Ratatui	8	Rendu en temps réel, codage couleur approprié.
- Qualité du Code	7	Structure propre, gestion d'erreurs appropriée, documentation.
Fonctionnalités Avancées	15	
- Optimisation	5	Pathfinding efficace, stratégies d'allocation des ressources.
- Robustesse	5	Gestion des cas limites (épuisement des ressources, évitement de collisions).
- Expérience Utilisateur	5	Simulation fluide, retour visuel clair.
