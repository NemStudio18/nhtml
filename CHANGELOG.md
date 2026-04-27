# 📜 Changelog NHTML

## v0.6.0 (Mai 2026) — "Global Connect" (EN DÉVELOPPEMENT)
### Added
- **FastCGI/PHP-FPM Bridge**: Support pour la communication directe avec les pools FPM (ultra-basse latence).
- **Scoped Broadcasting**: Possibilité de diffuser des patches à tous ou aux autres clients d'une session.
- **Industrial Error Handling**: Refonte totale des erreurs via `GatewayError` (zéro panique).

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
