---
# 🏆 NHTML Gateway — Master Plan de Remédiation & Roadmap
**Fichier d'audit** : `audit_19_master_plan.md`
**Date** : 29 Avril 2026

Ce document regroupe toutes les vulnérabilités identifiées lors de l'audit complet de la v0.7.3-beta, classées par ordre strict de priorité. Il propose également des pistes d'amélioration pour la performance pure et des idées de fonctionnalités novatrices pour l'avenir.

---

## 🛑 PARTIE 1 : Liste des vulnérabilités classées par priorité

### 🔴 CRITIQUES (Résolution immédiate)
> Impacte directement la sécurité des utilisateurs. Exploitation triviale.
*   **[CRIT-04-001] Faille XSS Stored / Reflected via `innerHTML` (`bridge.js`)**
    L'injection de contenu dynamique via `innerHTML` sans filtre permet à un attaquant d'exécuter des scripts malveillants. 
    **Fix** : Implémenter DOMPurify ou utiliser `.textContent` par défaut.

### 🟠 HAUTES (Résolution prioritaire)
> Peut permettre de contourner les mécanismes de sécurité du serveur.
*   **[HIGH-07-001] Contournement du CSWH (Cross-Site WebSocket Hijacking) (`socket/mod.rs`)**
    La vérification de l'Origin utilise `.contains(host)` (ex: `attaquant-monsite.com` passera).
    **Fix** : Utiliser un parsing d'URL strict et une égalité stricte du nom de domaine.

### 🟡 MOYENNES (Résolution sous 1 à 3 mois)
> Nuit à la stabilité globale, à l'intégrité des données ou introduit un risque ciblé.
*   **[MED-01-001] Dépendance vulnérable à "Marvin Attack" (`Cargo.toml`)**
    La crate `rsa` via `sqlx` (MySQL/Postgres) est obsolète. 
    **Fix** : Forcer la mise à jour des dépendances via `cargo update` ou mettre à jour la version de `sqlx`.
*   **[MED-09-001] Incohérence de données lors du TTL (`session.rs`)**
    Le processus de nettoyage des sessions expirées ne gère pas de transactions (risque de requêtes orphelines en cas de coupure). 
    **Fix** : Envelopper les `DELETE` dans un bloc `BEGIN ... COMMIT` ou ajouter `ON DELETE CASCADE` aux tables.
*   **[MED-16-001] Troncature silencieuse de la taille du B-TREE (`btree_builder.rs` / `proto.rs`)**
    Un attribut ou du texte trop long (>65Ko) corrompra le flux réseau à cause du cast aveugle `as u16`.
    **Fix** : Limiter la longueur avant l'encodage binaire ou basculer les paquets sur un `DataLen` en `u32` (4 Go).
*   **[MED-08-001] Déni de Service du Pool FPM (`socket/mod.rs`)**
    Si le backend PHP sature, le serveur ferme purement la connexion (`FpmPool saturé`) au lieu de la mettre en file d'attente asynchrone.
    **Fix** : Utiliser `tokio::sync::Semaphore` avec timeout pour créer une vraie file d'attente non-bloquante.
*   **[MED-02-001] Manque de limites et Headers par défaut (`nhtml.config.toml`)**
    Aucun rate_limit (hors mémoire) ni CSP activé de base.
    **Fix** : Ajouter des modèles de headers de sécurité activés par défaut lors du scaffold.
*   **[MED-13-001] Supply Chain via `npx localtunnel` (`cli.rs`)**
    La commande `nhtml share` exécute implicitement un package distant Node.
    **Fix** : Ajouter un prompt ou une confirmation de sécurité avant l'exécution.

### 🔵 QUALITÉ ET PERFORMANCE (À planifier dans les sprints)
*   **[PERF-09-001] N+1 Queries sur le Nettoyage (`session.rs`)**
    Boucle `for` avec une requête `DELETE` individuelle par session (asphyxie de la DB SQLite/MySQL sous forte charge).
*   **[PERF-08-001] Blocage du Thread Tokio par le Compilateur (`socket/mod.rs`)**
    La compilation `.nhtml` au sein du WebSocket est synchrone. À déléguer via `tokio::task::spawn_blocking`.
*   **[PERF-04-001] Layout Thrashing (Chute de FPS) (`bridge.js`)**
    Lecture synchrone de `getBoundingClientRect()` sur l'événement très fréquent `mousemove`.
*   **[QUAL-13-001] Rupture Multi-Plateforme (`cli.rs`)**
    L'appel système à `xcopy` casse la commande `build` sous Linux et macOS.

---

## ⚡ PARTIE 2 : Idées d'Optimisations Extremes (Performance)

Afin de positionner NHTML comme la solution temps réel la plus rapide du marché, voici des améliorations d'architecture :

1.  **Parsing Binaire "Zero-Copy" (Rust)** :
    Actuellement, `decoder.rs` clone les chaînes via `.to_string()`. Utiliser la crate `bytes` et le trait `Cow<'a, str>` permettrait au Gateway de router les paquets sans aucune réallocation mémoire sur le tas (Heap). Le coût CPU chuterait de 30% sur les nœuds très sollicités.
2.  **Mise en Cache "Edge" du compilé B-TREE** :
    Plutôt que de parser le `.nhtml` à la volée, le B-TREE complet pourrait être stocké dans une map en mémoire partagée (`moka` cache ou `dashmap`) et expédié directement. On économiserait le temps de parsing complet au moment du `HELLO`.
3.  **Transport UDP via WebTransport (HTTP/3)** :
    Remplacer progressivement les WebSockets (basés sur TCP/HTTP1.1) par WebTransport. Pour un framework visant l'échange de paquets très rapides, le mode "Unreliable" éviterait le phénomène du "Head-of-line blocking" classique des WebSockets, assurant une latence de l'ordre de la milliseconde pour les Local Actions (MouseMove/Hover).

---

## 🔮 PARTIE 3 : Fonctionnalités "Killer Features" à envisager (v1.x)

Maintenant que nous avons une infrastructure unifiée (Frontend JS / NHTML Gateway Rust / Backend PHP), l'horizon est grand ouvert :

1.  **Réconciliation Binaire Différentielle ("Delta Sync")** :
    Au lieu de re-télécharger tout le B-TREE lors d'une reconnexion (après une perte de signal sur mobile), le serveur NHTML ne pourrait renvoyer *que* les Opcodes PATCH depuis la dernière `last_seq`. Le client récupèrerait son contexte exact à la milliseconde près, sans blink (scintillement).
2.  **Mode "Offline-First" Transparent** :
    Utiliser le Service Worker via `bridge.js` pour capturer les requêtes (Events). Si le réseau coupe, les paquets binaires sont accumulés dans une file locale (`IndexedDB`). Dès la reprise, le SDK envoie tout le batch d'un coup au Gateway PHP pour un traitement en file indienne.
3.  **PHP Hot Module Replacement (HMR)** :
    Couplé au `watcher.rs` déjà implémenté, NHTML pourrait envoyer un paquet spécial `PKT_PATCH` qui ne recharge pas la page du navigateur, mais réinjecte uniquement les parties de la page modifiées par le script PHP tout en conservant le focus du curseur et l'état des formulaires du client. L'expérience développeur (DX) s'approcherait de Vite.js.
4.  **"WebAssembly Edge" (Zero-Server avancé)** :
    Puisque le protocole NBPS est déjà capable de fonctionner sans backend traditionnel (via l'Opcode `0x08 PUSH_PATCH`), NHTML pourrait encapsuler sa propre logique métier dans des modules WASM distribuables aux clients (navigateurs). La DB deviendrait un simple espace de stockage de CRDT (Conflict-free Replicated Data Types) pour des applications P2P collaboratives en pur JS/Rust.
