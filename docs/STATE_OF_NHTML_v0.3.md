# 🌐 NHTML : Spécification de Référence (v0.3.0)
**Date :** Avril 2026
**Statut :** 🟢 Stable (Réconciliation Code/Spec)

Ce document remplace la v0.2.3 et sert de source de vérité pour l'écosystème NHTML.

---

## 1. 🎯 Vision et Architecture "Hybrid-Native"

NHTML est un framework "Server-Driven UI" visant le **Zéro-JS métier**. 

### La Stratégie de Déploiement :
1.  **Couche Active (Validation) :** Un micro-polyfill `bridge.js` (~2ko) et un moteur WASM assurent le transport binaire et le patching du DOM dans les navigateurs standards.
2.  **Cible Long Terme :** Un fork Chromium intégrant nativement le protocole NBPS pour supprimer totalement le besoin du polyfill.

---

## 2. 🏛️ Les 3 Modes de Transport Officiels

Le protocole NBPS s'abstrait de la couche réseau pour s'adapter à l'hébergement :

| Mode | Transport | Serveur | Cas d'usage |
|------|-----------|---------|-------------|
| **Dédié** | WebSocket | Gateway Rust | Applications temps-réel, performance max. |
| **Mutualisé** | HTTP POST | PHP Standard | Hébergements classiques (OVH, etc.). |
| **Statique** | Local FFI | PHP WASM | Sites "Serverless" (GitHub Pages). |

---

## 3. 💾 Protocole NBPS v0.3 (Standardisé)

### Header de Paquet (Universal) :
`[Type: 1 octet] [PayloadLength: 4 octets (u32 BE)] [Payload...]`

### OpCodes (Types de Message) :
*   `0x01` **HELLO** : Handshake (Sync Session & Versions).
*   `0x02` **EVENT** : Client -> Serveur (Interactions utilisateur).
*   `0x03` **PATCH** : Serveur -> Client (Mutations du DOM).
*   `0x04` **BIND** : Serveur -> Client (Mapping ID/NID et Local Actions).
*   `0x07` **B-TREE** : Serveur -> Client (Snapshot complet du DOM).
*   `0x09` **RELOAD** : Serveur -> Client (Commande de rafraîchissement forcé).

### Détail du Paquet PATCH (0x03) :
`[OpCount: 2 octets (u16)]` suivi de N opérations :
`[TargetID: 2 octets] [OpType: 1 octet] [NodeVersion: 4 octets] [DataLen: 2 octets] [Data...]`

---

## 4. 🛠️ Écosystème DevTools & Time Travel

Les DevTools ne sont plus une option mais un composant cœur de la v0.3.

### Le Replay Engine (Moteur de Flux) :
*   **Persistance :** Toutes les interactions sont logguées dans une base SQLite `nhtml_sessions.db`.
*   **Dogfooding :** Le dashboard DevTools est lui-même écrit en NHTML.
*   **Hot Reload :** Déclenchement automatique du paquet `0x09` lors d'une modification de fichier source.

---

## 🗺️ Roadmap de la v0.3 vers la v1.0

- [x] Unification du protocole binaire (Rust/WASM/JS).
- [x] Dashboard DevTools fonctionnel avec Replay.
- [ ] Finalisation du SDK PHP pour le mode mutualisé (Fallback HTTP).
- [ ] Implémentation du moteur de "Pause" et "Rewind" dans le Replay Engine.
- [ ] Plugins d'autocomplétion pour les IDE (n-id, n-click).
