# 🌐 NHTML : Précisions Techniques (v0.3.1)
**Date :** Avril 2026
**Statut :** 🟢 Spécification Étendue

Ce document complète la v0.3.0 en apportant des précisions critiques sur le versioning et les modes de transport dégradés.

---

## 1. 🔢 Gestion du `NodeVersion` (Conflict Resolution)

Le champ `NodeVersion (u32)` dans les paquets `PATCH` et `B-TREE` sert à garantir l'intégrité de l'état du DOM.

### Règles d'Or :
1.  **Source de Vérité** : Seul le **Serveur** incrémente les versions.
2.  **Mise à jour Client** : Le client met à jour sa version locale d'un nœud uniquement après avoir appliqué avec succès un `PATCH` ou un `B-TREE`.
3.  **Gestion des Conflits** :
    *   **Si `Version_Recue == 0`** : Le patch est appliqué sans vérification (Mode Compatibilité).
    *   **Si `Version_Recue <= Version_Locale`** : Le patch est considéré comme obsolète (lié à une action passée déjà traitée) et est **ignoré**.
    *   **Si `Version_Recue > Version_Locale + 1`** : Indique une perte de paquets. Le client doit invalider son cache local et peut déclencher une demande de `B-TREE` complet.

---

## 2. 🔌 Limitations du Mode Mutualisé (HTTP POST)

Le mode "Mutualisé" est une solution de secours pour les environnements sans WebSocket.

### Architecture Request/Response :
Contrairement au WebSocket (Full-Duplex), le mode HTTP POST suit un cycle strict :
`Client (EVENT) ---> Serveur (Traitement) ---> Client (PATCH)`

### Limitations Officielles :
*   **Pas de Push Serveur** : Le serveur ne peut pas envoyer de données au client de manière spontanée (ex: notification, mise à jour de log).
*   **Désactivation de `n-live`** : Les mises à jour automatiques basées sur des timers serveurs ne fonctionnent pas.
*   **Latence** : Chaque interaction nécessite l'ouverture d'une nouvelle connexion TCP/HTTPS (plus lourd que les frames WebSocket).

---

## 3. 🛡️ Integrated Supervisor (Auto-PHP)

Le Gateway intègre désormais un gestionnaire de processus pour le backend PHP.

### Logique de Détection :
Le Supervisor cherche le binaire PHP dans cet ordre de priorité :
1.  `./php.exe` (Dossier racine)
2.  `./php/php.exe` (Dossier d'empaquetage standard)
3.  `./bin/php.exe`
4.  `PATH` système (Commande `php`)

### Lifecycle :
*   Le serveur PHP est lancé sur le port 8000 par défaut.
*   Le processus est lié au Gateway : si le Gateway reçoit un signal `SIGINT` (Ctrl+C), il tue proprement le processus PHP pour éviter les ports orphelins.

---

## 4. 📊 Packet HUD & Visual Feedback

Le `bridge.js` inclut désormais des outils de diagnostic natifs activés en mode développement.

### NHTML Glow :
*   Chaque élément modifié par un `PATCH` reçoit temporairement la classe `.nhtml-patch-glow`.
*   Cela déclenche une animation de pulsation verte (CSS Animation) pour confirmer la réception de la mutation.

### Packet HUD :
Un overlay discret en bas à droite affiche :
*   **pkts** : Compteur total de paquets reçus.
*   **size** : Poids total des données transférées.
*   **mode** : Mode de transport actif (`WS` ou `HTTP`).

---

## 5. 📝 Nouveau Message : PKT_LOG (0x10)

Pour finaliser la Phase 2 de la Roadmap, le protocole intègre désormais le support des logs serveurs.

*   **OpCode** : `0x10`
*   **Direction** : Serveur -> Client (Uniquement en mode Dédié/WASM)
*   **Payload** : `[Severity: 1 octet] [MessageLen: 2 octets (u16)] [Message: UTF-8]`
*   **Usage** : Permet au backend (PHP/Rust) d'envoyer des informations de debug directement dans le "PHP Log Bridge" du navigateur.

---

## 4. 🗺️ Synthèse des OpCodes v0.3.1

| Code | Nom | Direction | Description |
|------|-----|-----------|-------------|
| `0x01` | HELLO | Bidirectionnel | Handshake / Sync Session. |
| `0x02` | EVENT | Client -> Srv | Interaction utilisateur (Clic, Input). |
| `0x03` | PATCH | Srv -> Client | Mutation du DOM (Text, Attr, Style). |
| `0x04` | BIND | Srv -> Client | Local Actions & ID Mapping. |
| `0x07` | B-TREE | Srv -> Client | Snapshot DOM complet. |
| `0x09` | RELOAD | Srv -> Client | Hot Reload Dev. |
| `0x10` | LOG | Srv -> Client | Debug Bridge (Nouveau). |
