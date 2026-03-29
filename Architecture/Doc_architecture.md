# Architecture de Simulation : Collecte de Ressources

Ce document décrit l'architecture technique du projet de simulation de robots autonomes, basée sur Rust et Ratatui. L'architecture s'appuie sur le **Modèle d'Acteur** pour garantir la concurrence et la fluidité de la simulation.

## Diagramme d'Architecture

![Alt text](https://raw.githubusercontent.com/DanielDSM23/Rust-Resource-Collection-Simulation/refs/heads/main/Architecture/images/diagram.png "diragram")

---

## Explication des Flux Principaux

### 1. Communication MPSC (Multi-Producer, Single-Consumer)
C'est le cœur de la coordination du système. 
* **Producteurs :** Tous les robots (éclaireurs et collecteurs) agissent comme des producteurs. Ils envoient continuellement des messages contenant leurs découvertes (nouvelles ressources, obstacles) et leurs actions en cours via des transmetteurs.
* **Consommateur :** Le Système de Base central agit comme l'unique consommateur. Il écoute le canal de réception et met à jour l'état global du monde en fonction des messages reçus.
* **Bénéfice :** Cette approche asynchrone permet à la base d'agréger les données sans jamais bloquer l'exécution des threads des robots.

### 2. Découplage de l'Interface Utilisateur (UI) et de la Simulation
L'Interface Utilisateur et la Logique de Simulation s'exécutent de manière totalement indépendante.
* **Thread UI (Ratatui) :** Dédié exclusivement à la gestion des événements clavier (pour quitter) et au rendu visuel. Il accède à l'État Global uniquement en **lecture seule** (par exemple, via un verrouillage léger de type `Arc<RwLock<T>>`).
* **Bénéfice :** Garantit un rendu terminal fluide et constant (ex: 30 ou 60 FPS). Si l'algorithme de recherche de chemin (*pathfinding*) d'un robot prend quelques millisecondes de trop, l'interface graphique ne sera pas figée pour autant.

### 3. Autonomie des Robots
* **Threads Indépendants :** Chaque robot fonctionne dans son propre thread (ou tâche asynchrone) avec sa propre boucle logique d'exploration ou de collecte.
* **Connaissances Limitées :** Les robots ne possèdent pas une vision globale et omnisciente de la carte. Ils maintiennent uniquement un état local basé sur leur environnement immédiat et les informations que la Base a partagées avec eux.
* **Bénéfice :** Respecte strictement l'exigence d'avoir des entités indépendantes, forçant l'utilisation de la communication distribuée pour accomplir les tâches de collecte de manière coordonnée.
