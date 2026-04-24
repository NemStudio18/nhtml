# 🌐 NHTML : Le Bilan Complet du Projet (v0.2.3)
**Date de révision :** Avril 2026
**Statut global :** 🟢 Alpha Avancée (Core et DevTools fonctionnels)

Ce document est la "Bible" actuelle du projet NHTML. Il résume la vision, la technique, le protocole, l'architecture de déploiement et l'avancement exact de la roadmap à ce jour.

---

## 1. 📖 Le Concept NHTML (Guide Rapide)
**NHTML (Native HTML)** est un framework qui vise à éliminer le Javascript métier côté frontend.
Plutôt que d'écrire des composants React/Vue, le développeur écrit du HTML augmenté (`n-click`, `n-model`) et toute la logique métier s'exécute côté serveur (actuellement en PHP ou Rust). 

**Comment ça marche ?**
1. Un micro-polyfill JS de ~2ko (`bridge.js`) est chargé dans le navigateur.
2. Il convertit toutes les interactions de l'utilisateur (clics, frappes au clavier) en **octets binaires purs**.
3. Il envoie ces octets par WebSocket au serveur.
4. Le serveur traite la demande et renvoie un **Patch binaire** très léger.
5. Le Polyfill applique la modification au DOM à la vitesse de la lumière.

---

## 2. 🏛️ Architectures de Déploiement

NHTML n'impose pas une architecture serveur stricte. Il s'adapte à 3 environnements :

1. **Mode "Serveur Dédié" (Actif 🟢)** : 
   Utilisation du Gateway Rust `nhtml`. Il gère les WebSockets asynchrones de façon ultra-performante et transfère les ordres au code PHP en local. Parfait derrière un proxy Nginx.
2. **Mode "Hébergement Mutualisé OVH" (En cours 🟡)** :
   Si le serveur n'autorise pas les WebSockets ni les binaires Rust, le Polyfill dégrade sa connexion en HTTP standard. Les événements binaires sont envoyés par des requêtes POST classiques traitées par un simple script PHP.
3. **Mode "Statique & WASM" (Futur ⚪)** :
   Pour des sites hébergés sur GitHub Pages (sans serveur). Le navigateur du visiteur télécharge un moteur PHP compilé en WebAssembly (WASM) et exécute le code serveur *localement* dans le navigateur.

---

## 3. 💾 Le Protocole Binaire NBPS (NHTML Binary Protocol System)
Le cœur de la performance de NHTML réside dans son protocole binaire (sur WebSocket ou HTTP).

### Les OpCodes principaux (Types de Paquets) :
* `0x01` **HELLO** : Handshake d'initialisation. Échange des versions et synchronisation de l'UUID de Session.
* `0x02` **EVENT** : Le client informe le serveur qu'une action a eu lieu (ex: Clic sur le Node `1005`).
* `0x03` **PATCH** : Le serveur ordonne au client de muter le DOM (ex: `SET_TEXT`, `SET_ATTR`, `ADD_CLASS`).
* `0x04` **BIND** : Le serveur envoie les *Local Actions* au client (animations CSS, hovers) pour soulager le réseau.
* `0x07` **B-TREE** : Envoi d'un snapshot complet de l'arbre DOM pour initialisation massive.
* `0x09` **RELOAD** : Commande système (Hot Reload lors du développement).

---

## 4. 🛠️ L'Écosystème DevTools (v0.2.3)
Le Gateway Rust intègre des outils de classe mondiale pour les développeurs :

- **Le Time Travel (Replay Engine)** : Chaque interaction de l'utilisateur est historisée dans une base de données SQLite (`nhtml_sessions.db`, limitée aux 1000 dernières actions pour éviter la surcharge).
- **Le Dashboard DevTools** : Accessible via la commande `nhtml devtools` (sur le port 8081).
  - Il est **100% construit en NHTML** (principe de "Dogfooding").
  - Il permet de sélectionner une ancienne session dans une liste.
  - Il inclut une barre temporelle pour rejouer l'état de l'application action par action.
- **L'Inspecteur CLI** : Commande `nhtml inspect <hex>` pour traduire les flux binaires bruts en JSON lisible par l'humain.
- **Le Hot Reload** : Le serveur surveille les fichiers `.php` et `.nhtml` et demande au navigateur de se rafraîchir à la moindre sauvegarde.

---

## 5. 🗺️ État d'Avancement et Roadmap

### ✅ Phase 1 : Fondations du Protocole (100% 🟢)
- Polyfill JS (bridge.js) finalisé et robuste.
- Sérialisation/Désérialisation binaire parfaite en Rust (`proto.rs`).
- Connectivité WebSocket bidirectionnelle stable.

### ✅ Phase 2 : Industrialisation et DevTools (90% 🟢)
- Système de Sessions UUID persistant.
- Intégration de SQLite et Event Logging.
- DevTools Dashboard (Sélection de session).
- Replay Engine (Moteur de retour dans le temps).
*Reste à faire : Ajout du mode Reverse (Pause/Recul dans le Replay Engine).*

### 🚧 Phase 3 : SDKs & Modes de Déploiement (30% 🟡)
- SDK PHP binaire opérationnel.
*Reste à faire : Finaliser le fallback HTTP (pour les hébergements mutualisés sans Rust).*
*Reste à faire : Plugins IDE (Autocomplétion VSCode).*

### ⏳ Phase 4 : Production & Marketing (10% ⚪)
*Reste à faire : Rédaction finale de la Landing Page.*
*Reste à faire : Compilation des binaires officiels (Mac, Win, Linux) pour le grand public.*
*Reste à faire : Tutoriel interactif de démarrage.*
