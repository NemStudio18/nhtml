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
