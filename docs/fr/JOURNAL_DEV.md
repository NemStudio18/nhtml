# 📒 Journal de Bord - NHTML DevTools Industrialization

## 📅 2026-04-24 - Session Matin
### 🛠️ Modifications Structurelles
1.  **`src/main.rs` (MPSC Refactor - STABLE)** :
    *   **Architecture** : Remplacement de l'approche `Arc<Mutex<SplitSink>>` par un système de canaux **MPSC** (Multi-Producer Single-Consumer) par client.
    *   **Writer Task** : Une tâche dédiée possède désormais l'accès exclusif au `ws_sender`, éliminant les erreurs de traits `SinkExt` et les verrous complexes.
    *   **Log Relay** : Les logs PHP sont injectés dans ce canal MPSC de manière asynchrone et thread-safe.
    *   **Reload Support** : Les signaux du Watcher passent aussi par ce canal.
2.  **`src/supervisor.rs`** :
    *   Désormais responsable de la capture de `stdout`/`stderr` du processus PHP.
    *   Diffuse les lignes de logs via deux canaux : `tx_monitor` (Dashboard) et `tx_app_broadcast` (App clients).
3.  **`static/devtools.nhtml`** :
    *   Refonte de l'UI (Onglets SESSIONS / NETWORK).
    *   Ajout du **Payload Tester** (Injection manuelle de paquets).
    *   Ajout d'explications contextuelles pour chaque outil.
4.  **`counter/polyfill/bridge.js`** :
    *   Restauration de la **Verbosité Protocol** (Logs HELLO, IN/OUT, SID).
    *   Fix du chemin d'import (pointant vers le port 8000 en absolu).
    *   Mirroring des logs console (F12) fonctionnel via l'OpCode `0x10`.

### 📍 Points d'Ancrage Actuels
*   **Port 8000** : Serveur PHP (Supervisé par le Gateway).
*   **Port 8080** : Gateway NBPS (Application). Supporte désormais `0x0B` (APPEND_HTML).
*   **Port 8081** : Dashboard DevTools Pro (v0.3.1).
*   **Base de données** : `./nhtml_sessions.db` (Table `patch_history`).
*   **Protocol** : v0.3.1 unifié (Headers 5 octets).

### 🚀 Prochaines Étapes (Phase 4)
*   Implémenter le rapport de compression B-TREE.
*   Finaliser le packaging v0.4.0.
*   Génération de la documentation technique de référence (Code-only analysis).

## 📅 2026-04-26 - Résolution des Ruptures (Audit v0.4.0)
### 🛠️ Corrections Apportées
1. **[RUPTURE-06] & [RUPTURE-05]** : Ajout du listener `keydown` dans `bridge.js` et implémentation du parseur et runner pour les *Local Actions* (`n-hover`, `n-toggle`, `n-scroll`, etc.) sans latence serveur.
2. **[RUPTURE-01] & [RUPTURE-02]** : Reconnexion du superviseur PHP dans le flux principal de `src/main.rs`. Le serveur de DevTools (port 8081) démarre désormais en même temps que la gateway ou via la commande CLI `nhtml devtools`.
3. **[RUPTURE-08]** : Ajout de l'appel à `session_manager.register_session()` lors du handshake WebSocket dans `src/socket/mod.rs` pour peupler la table `sessions` de SQLite.
4. **[RUPTURE-03]** : Activation de la compression Zstd (niveau 3) sur les paquets B-TREE. La compression n'est appliquée que si elle réduit réellement la taille du payload (`comp_flag = 0x01`).
5. **[RUPTURE-04]** : Mise en place d'une tâche périodique (toutes les 30s) avec `tokio::select!` dans la boucle WebSocket pour calculer et envoyer le checksum (paquet `SYNC`) du DOM au client.
6. **[RUPTURE-09]** : Nettoyage de `src/core.rs` pour retirer le code mort (doublons `PatchOp`, `Node`, `EventLogEntry`) hérité de la v0.2.x, en ne gardant que le `SessionState`.

### [2026-04-26] Stabilisation DevTools & Monitoring Final
7. **Monitoring Event Relay** : Implémentation de `monitor_pkt` dans `src/socket/mod.rs`. Le Gateway diffuse désormais chaque paquet (IN/OUT, type, taille, handler) sur le canal `tx_monitor`, permettant au dashboard DevTools (port 8081) d'afficher le trafic en temps réel.
8. **DevTools Logic Fix** : Le serveur DevTools dans `src/cli.rs` sert désormais lui-même le `bridge.js` sur `/_nhtml/bridge.js` pour éviter les erreurs de chargement inter-origines ou d'absence du fichier.
9. **Bridge.js Port Auto-detect** : Correction de l'auto-initialisation dans `bridge.js` pour utiliser `window.location.port`, permettant au client de se connecter automatiquement soit à la Gateway (8080), soit au DevTools (8081) selon l'URL chargée.
10. **Type Safety** : Correction de la signature de `call_php` pour correspondre au retour de `parse_php_response` (support du flag broadcast). Nettoyage exhaustif des warnings de compilation.

### [2026-04-26] Stabilisation Industrielle & Fixes Bug (v0.4.0 Final)
11. **Binary Parsing Fix (bridge.js)** : Correction critique du calcul d'offset pour les chaînes UTF-8. Utilisation de `TextEncoder` pour lire la taille réelle en octets (`tagLen`, `valLen`) au lieu du nombre de caractères, éliminant les erreurs `querySelector` sur IDs corrompus.
12. **Id Numeric Fallback** : Modification de `bridge.js` pour permettre la résolution directe des `id_num` numériques passés en `n-id` (cas des éléments générés dynamiquement par DevTools comme les boutons "Charger").
13. **Source ID Resolution (Rust)** : Le Gateway Rust résout désormais le `n-id` textuel (ex: `slider_radius`) avant d'appeler PHP, permettant aux exemples complexes comme le **Style Lab** de fonctionner sans modification du code backend.
14. **Chat Logic Filter** : Mise à jour de `app.php` dans l'exemple Chat pour ignorer les événements `keydown` ne correspondant pas à la touche `Enter`, réduisant le trafic inutile.
15. **DevTools Comparison (CMP)** : Implémentation du backend pour l'OpCode `0x04` dans `cli.rs`, permettant la comparaison d'état entre deux sessions.
16. **Multi-root Body Support** : Correction majeure du compilateur Nhtml pour parser tous les enfants du `<body>`. Introduction d'un "Virtual Body Root" permettant la synchronisation B-TREE et BIND de plusieurs éléments racines (cas du Style Lab).
17. **Bridge Robustness** : Nettoyage syntaxique de `bridge.js` (suppression de doublons, ajout de try-catch sur `processMessage`) pour garantir la stabilité du client face à des paquets malformés ou des erreurs de sélection DOM.

**État Final v0.4.0-STABLE** : Écosystème validé à 100%. Style Lab, Chat et DevTools pleinement opérationnels avec une gestion binaire rigoureuse.
