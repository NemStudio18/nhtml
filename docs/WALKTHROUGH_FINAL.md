# Walkthrough : Industrialisation & Écosystème (v0.3.1)

Ce document résume les étapes techniques franchies pour stabiliser l'écosystème NHTML en version **v0.3.1**.

## 🏁 1. Consolidation et Protocol v0.3.1
- **Headers Universels** : Tous les paquets utilisent désormais un header `[Type:1][Len:4]` (u32 BE).
- **NodeVersion** : Introduction du versioning des nœuds (u32) pour éviter les désynchronisations.
- **OpCodes Fixés** : `0x02` (EVENT), `0x03` (PATCH), `0x09` (RELOAD), `0x10` (LOG).

## 🐘 2. Supervisor & Auto-PHP
Le Gateway Rust gère maintenant le cycle de vie de PHP :
- **Auto-Détection** : Recherche intelligente du binaire `php` (Local, dossier `php/`, ou PATH).
- **Lancement Transparent** : Le serveur PHP est lancé automatiquement sur le port 8000 avec le `router.php`.

## 🛠️ 3. Visual Stack & Diagnostic (Phase 1 & 2 - 100%)
- **NHTML Glow** : Flash visuel vert sur les éléments du DOM lors d'un `PATCH`.
- **Packet HUD** : Bulle d'info in-page affichant le nombre de paquets, le débit et le mode de transport.
- **PHP Log Bridge** : Redirection des logs serveur directement dans la console F12 via le paquet `0x10`.

## 🛰️ 4. DevTools Pro & Transport Hybride (Phase 3 - 80%)
- **Network Monitor** : Flux live des paquets NBPS dans l'onglet "NETWORK LIVE".
- **Node Inspector** : Cliquer sur le Replay pour inspecter l'état binaire d'un noeud.
- **Fallback HTTP** : Bascule automatique sur `fetch` POST en cas d'échec WebSocket.
- **Dual-Mode HUD** : Affichage temps-réel du mode de transport sur le client.

---
**Status Final :** L'écosystème **v0.3.1** est stabilisé et industrialisé. Le Gateway est désormais un outil de diagnostic complet et le protocole est prêt pour un déploiement agnostique (WS/HTTP).
