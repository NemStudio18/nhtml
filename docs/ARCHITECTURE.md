# 🛰️ Référence Technique NHTML (v0.3.1)
**Document généré par analyse directe du code source.**

---

## 1. Vue d'ensemble du projet
NHTML (Native-HTML) est un framework de développement web "Server-Driven" conçu pour déporter toute la logique métier et l'état de l'application côté serveur tout en offrant une réactivité proche du client natif.

- **Objectif** : Éliminer la complexité du JS frontend en transformant le navigateur en un simple moteur de rendu binaire piloté par un backend (PHP/Rust).
- **Cas d'usage** : Applications temps réel, interfaces industrielles, tableaux de bord de monitoring.
- **Stack Technique** :
    - **Gateway** : Rust (Tokio, Axum/Tungstenite).
    - **Backend** : PHP 8.x (Mode supervisé).
    - **Client** : Vanilla JS (Moteur binaire NBPS).
    - **Base de données** : SQLite (Sessions & Métier).

---

## 2. Architecture Générale
L'architecture repose sur un **Gateway Orchestrateur** agissant comme un pont binaire bidirectionnel.

```mermaid
graph TD
    Browser[Navigateur (bridge.js)] <-->|NBPS (Binary/WS)| Gateway[Gateway Rust]
    Gateway <-->|JSON/HTTP| PHP[App PHP (SDK)]
    Gateway -->|Broadcast| DevTools[Dashboard DevTools 8081]
    Gateway <-->|SQLite| DB_Sessions[nhtml_sessions.db]
    PHP <-->|SQLite| DB_App[app.db]
```

---

## 3. Structure des Répertoires Clés
- `/gateway/src/` : Cœur du système Rust.
    - `main.rs` : Orchestration des services et boucle de connexion WS.
    - `proto.rs` : Source de vérité du protocole binaire (S/D).
    - `supervisor.rs` : Gestionnaire de processus pour le serveur PHP.
    - `session.rs` : Gestionnaire de persistance SQLite pour le monitoring.
- `/gateway/counter/` : Application de démonstration.
    - `app.php` : Logique métier (Incrémentation).
    - `index.nhtml` : Structure DOM pilotée par le serveur.
- `/gateway/counter/polyfill/` : Moteur client.
    - `bridge.js` : Client WebSocket et processeur de paquets binaires.
- `/sdk/php/` : Abstraction pour les développeurs backend.

---

## 4. Protocole NBPS v0.3.1 (Native-HTML Binary Protocol)
Le protocole est optimisé pour la bande passante et la latence.

- **Format** : Big-Endian.
- **Header Universel (5 octets)** : `[Type: u8] [Length: u32]`.
- **OpCodes Principaux** :
    - `0x01` (HELLO) : Synchronisation initiale de session.
    - `0x02` (EVENT) : Notification d'interaction client vers serveur.
    - `0x03` (PATCH) : Instructions de mutation DOM atomiques.
    - `0x07` (B-TREE) : Snapshot complet pour la resynchronisation.
    - `0x10` (LOG) : Flux de débogage backend vers console client.

---

## 5. Modèle de Données
Le système utilise deux couches de persistance SQLite distinctes :

### A. nhtml_sessions.db (Gateway)
- **Table `patch_history`** : Stocke chaque mutation envoyée pour permettre le "Replay" et le diagnostic.
- **Colonnes** : `session_id`, `node_id`, `version`, `data` (BLOB).

### B. counter.db (Application)
- **Table `state`** : Persistance métier pure.
- **Colonnes** : `id`, `counter_value`.

---

## 6. Flux Fonctionnel : Le Cycle de Vie d'un Clic
1. **Captation** : `bridge.js` intercepte un clic sur un élément `n-click`.
2. **Émission** : Envoi d'un paquet `0x02` (EVENT) au Gateway (Port 8080).
3. **Traduction** : Le Gateway convertit l'EVENT en requête JSON POST vers PHP.
4. **Traitement** : `app.php` incrémente la valeur en BDD et génère une liste de "Patches" JSON.
5. **Conversion** : Le Gateway sérialise les patches JSON en binaire `0x03` (PATCH).
6. **Rendu** : `bridge.js` reçoit le binaire, applique les mutations au DOM et déclenche une animation (Glow).

---

## 7. Configuration & Déploiement
### Prérequis :
- **Rust** (Edition 2021).
- **PHP 8.x** (doit être dans le PATH ou dans `./php/`).

### Commandes de lancement :
```bash
# Lancement complet (Gateway + PHP + DevTools)
cargo run start --dev

# Accès :
# App : http://127.0.0.1:8000/counter/index.nhtml
# Dashboard : http://127.0.0.1:8081
```

---

## 8. Points Critiques & Dette Technique
- **Gestion du Lag** : Le système utilise des canaux `broadcast` de Tokio. Un mécanisme de "Lag handling" est présent dans `main.rs` pour éviter que les clients lents ne bloquent le flux global.
- **Séquençage** : Le `NodeVersion` garantit que les patches ne sont pas appliqués dans le désordre, mais une perte de paquet majeure nécessite un rechargement (`0x09`).
- **TODO** : La compression Zstd pour les snapshots B-TREE est définie dans le protocole mais reste à activer dans le moteur Rust.

---

## 9. Glossaire Technique
- **N-ID** : Identifiant unique d'un nœud DOM persisté côté serveur.
- **Patch Glow** : Feedback visuel signalant une mise à jour réactive.
- **NodeVersion** : Compteur d'évolution d'un nœud pour la résolution de conflits.
- **Supervisor** : Composant Rust pilotant le cycle de vie du processus PHP.
