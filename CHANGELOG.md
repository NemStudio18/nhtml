# 📜 Changelog NHTML

## v0.7.0 (Juin 2026) — "Industrial Scale" (DEVELOPMENT)
### Added
- **Redis Gateway Clustering**: Support natif pour le déploiement multi-nœuds avec synchronisation des sessions via Redis.
- **Filtering Multi-Gateway**: Prévention des boucles de broadcast via filtrage par `gateway_id` unique.
- **Observability Dashboard**: Nouveaux onglets FLOW, METRICS et SESSIONS dans les DevTools.
- **Prometheus Export**: Exportation des métriques système (Clients actifs, Débit paquets) vers Prometheus/Grafana.
- **Structured JSON Logging**: Flag `--json` pour intégration avec ELK et Datadog.
- **Triple-License Model**: MIT (SDKs), AGPLv3 (Gateway Core), et Licence Commerciale.

### Fixed
- **Security Hardening**: Correction d'une faille de Path Traversal dans le serveur HTTP.
- **HMAC Reliability**: Correction de l'offset de signature et couverture totale du payload.
- **Memory Safety**: Prévention des fuites mémoire dans le Rate Limiter et protection contre les overflows.
- **Port Handling**: Suppression d'un panic possible sur les ports invalides.
- **Unix Compatibility**: Correction d'une erreur de compilation sur les sockets Unix.

---

## v0.6.0 (Mai 2026) — "Global Connect" (FINALISÉ)
### Added
- **FastCGI/PHP-FPM Bridge**: Support pour la communication directe avec les pools FPM (ultra-basse latence).
- **Scoped Broadcasting**: Possibilité de diffuser des patches à tous ou aux autres clients d'une session.
- **Industrial Error Handling**: Refonte totale des erreurs via `GatewayError` (zéro panique).
- **IP-Based Rate Limiting**: Protection native contre le spam d'événements basée sur l'adresse IP réelle des clients.
- **Native TLS Support**: Support direct HTTPS/WSS via Rustls (native).
- **Industrial Benchmark Tool**: Métriques de performance avancées (Latence économisée, complexité CPU, gain de bande passante).
- **Production Build Engine**: Commande `nhtml build --production` avec minification et packaging NBPS optimisé.
- **Project Sharing**: Commande `nhtml share` pour exposer un projet localement via tunnel sécurisé.
- **Integration Test Suite**: Couverture de tests robuste pour l'alignement du protocole et la logique gateway.

---

## v0.5.0 (Avril 2026) — "Industrial Security"
### Added
- **HMAC-SHA256 Signatures**: Authentification binaire forcée pour chaque événement client.
- **Sequence ID Synchronization**: Protection native contre les attaques par rejeu.
- **Persistence WASM (IDBFS)** : Les données Messenger/Settings persistent désormais localement en mode WASM.
- **Responsive Showcase**: Layout adaptatif (Flex Desktop / Sidebar Mobile) pour la démo phare.

### Improved
- **Bridge Reliability**: Les événements sur les éléments imbriqués ne sont plus bloqués par les masques parents.
- **Documentation**: Mise à jour complète de l'architecture et de la sécurité.

---

## v0.4.0 (April 2026) — "Industrial Launch"
### Added
- **Multi-Platform Releases**: Automated builds for Windows, Linux, and macOS.
- **Premium Showcase MVC**: A complete flagship application with real-time inventory and dashboard.
- **Bilingual Documentation**: Full English and French documentation suite.
- **Compression Zstd (Active)**: B-TREE snapshots are now compressed by default (~80% size reduction).
- **Auto-Injection Bridge**: Gateway automatically injects `bridge.js` + `fzstd.js` into `.nhtml` files.
- **Integrated DevTools**: Real-time packet inspection and session time-travel replay.
- **SQLite Persistence**: Local database support for session management and showcase data.

### Improved
- **NBPS v0.4.0 Protocol**: Universal 5-byte header and strict version tracking for atomic patches.
- **Performance**: DOM mutations now processed in <1ms on the client side.
- **Security**: AGPL-3.0 License applied across the entire ecosystem.

---

## v0.3.1 (April 2026)
### Breaking Changes
- **Header Upgrade**: 3 bytes → 5 bytes (`Length` is now `u32`).
- **OpCode Realignment**: Standardized operation codes for the NBPS protocol.

### Added
- **0x10 LOG**: Binary log relay from server to browser console.
- **Supervisor**: Gateway now automatically manages the PHP backend lifecycle.

---

## v0.2.x
- **Baseline**: Initial binary NBPS protocol support.
