# 📜 Changelog NHTML

## v0.7.3 (Avril 2026) — "Scale & Observability" (WIP)
### Added
- **Native Load Balancing**: Dispatcher intelligent intégré supportant les stratégies **Round-Robin** et **Least-Connections**.
- **Automated Healthchecks**: Surveillance active des backends FPM avec mise en quarantaine automatique des nœuds défaillants.
- **Multi-Backend Configuration**: Support pour plusieurs adresses FastCGI/FPM simultanées via `nhtml.config.toml`.
- **Evolution Branding**: Intégration du nouveau concept de logo minimaliste symbolisant l'évolution du HTML.
- **Config Sections**: Ajout des sections `[security]` et `[cluster]` (avec auth Redis commentée) dans le template `nhtml.config.toml`.
- **CI Cache**: `Swatinem/rust-cache@v2` ajouté dans `release.yml` pour accélérer les builds CI de ~60%.
- **Test Suite v0.7.3**: Nouveau fichier `tests/v0_7_3_hardening.rs` couvrant la profondeur récursive, les paquets proto, et les bounds du B-TREE.

### Fixed
- **Session Storage**: `localStorage` migré vers `sessionStorage` pour le `session_id` dans `bridge.js` (expiration automatique à la fermeture de l'onglet).
- **WASM Security**: La fonction `switchToWasm()` est désormais conditionnée à un contexte HTTPS en production (bloque les fallback non sécurisés).
- **Compiler DoS**: Limite de profondeur récursive (max **500 niveaux**) ajoutée au compilateur NHTML (`compiler/mod.rs`) pour prévenir tout stack overflow.
- **Redis Auth Warning**: Le bridge cluster (`cluster.rs`) émet désormais un `WARN` explicite si l'URL Redis ne contient pas de credentials en production.
- **CI Pinning**: `softprops/action-gh-release` épinglé par SHA dans `release.yml` (protection Supply Chain).
- **Docs v0.7.3**: Synchronisation complète des docs EN/FR (`ARCHITECTURE.md`, `SECURITY.md`, `SECURITE.md`) avec les fonctionnalités de la v0.7.3.

## v0.7.1 (Avril 2026) — "Industrial Hardening" (ACTUEL)
### Added
- **Agnostic Database Engine (sqlx)**: Support complet pour **SQLite**, **MySQL** et **PostgreSQL** via `sqlx::AnyPool`.
- **Global Session Persistence**: Centralisation de l'état des sessions et de l'historique des patches en DB, permettant la scalabilité horizontale (Clustering).
- **Compilation Caching**: Cache global des `CompileResult` via `Arc`. Latence réduite à ~1ms lors des reconnexions.
- **Origin Validation**: Protection contre le *Cross-Site WebSocket Hijacking* (CSWH) via contrôle de l'en-tête Origin.
- **Memory Optimization**: Migration vers `Arc<HandlerTable>` et pré-sérialisation JSON pour réduire drastiquement l'empreinte mémoire par session.

### Fixed
- **Zero-Panic Compliance**: Suppression totale des `unwrap()` et `expect()` dans le runtime (Supervisor, Watcher, CLI, Socket).
- **Binary Hardening**: Ajout de `bounds checks` rigoureux sur tous les buffers entrants (NBPS v0.7.1).
- **HMAC Signatures**: Correction de l'implémentation binaire pour une compatibilité parfaite avec les navigateurs modernes.
- **PHP Path Security**: Canonicalisation forcée des chemins pour neutraliser les injections de commandes.
- **WASM MIME Support**: Support explicite des types MIME `.mjs` et `.wasm` pour le mode Zero-Server.
- **SDK Standardisation**: Alignement complet des SDK Python, Node.js et Go sur le modèle PHP (Broadcast Scopes, Parsing NBPS v0.7).
- **UI Industrialization**: Remplacement de tous les emojis par des icônes Font Awesome professionnelles et intégration du nouveau logo premium.
- **XSS Prevention (Bridge)**: Injection automatique de DOMPurify et sécurisation de `innerHTML`/`insertAdjacentHTML` contre les payloads malveillants.
- **DevTools Security**: Échappement HTML strict via `html_escape::encode_safe` sur les noms des handlers et payloads envoyés aux DevTools.
- **CLI Scaffolding**: Correction du template `app.php` généré par `nhtml new` pour supporter le format JSON via `php://stdin`.
- **Router Hardening**: `router.php` n'inclut désormais explicitement que des fichiers nommés `app.php` et masque son arborescence système (404 générique).
- **CSWH Protection**: Comparaison stricte des Origins WebSocket pour empêcher le Cross-Site WebSocket Hijacking.
- **Configuration Safety**: Le Gateway s'arrête avec une erreur fatale propre (exit 1) si `nhtml.config.toml` est invalide au lieu d'utiliser un fallback silencieux.
- **Port Cascading**: L'argument CLI `--port` prend désormais correctement le pas sur la configuration du fichier.
- **DevTools Access**: Accès restreint uniquement au mode développement (`--dev`) et protégé par un Token UUID aléatoire à chaque lancement.
- **Credential Masking**: Masquage automatique des mots de passe (`***`) pour la variable `NHTML_DB_URI` dans les logs terminaux.
- **Cross-Platform CLI**: Remplacement de l'appel système `xcopy` par une fonction récursive `copy_dir_all` 100% native Rust.
- **CLI Security**: Ajout d'une confirmation interactive (`[y/N]`) avant d'exécuter `npx localtunnel` (Supply Chain).
- **Transaction Safety**: Sécurisation transactionnelle (`BEGIN/COMMIT`) du nettoyage TTL des sessions en DB (`session.rs`).
- **B-TREE Integrity**: Limitation sécurisée à 64Ko avec préservation des frontières de caractères UTF-8 pour éviter le crash client sur des nœuds immenses.
- **Queue System (DoS Prevention)**: Implémentation d'une file d'attente non-bloquante (`tokio::sync::Semaphore`) pour le Pool FPM afin d'éviter la fermeture prématurée des requêtes en charge forte.
- **Dependency Update**: Résolution de la faille de sécurité "Marvin Attack" via mise à jour du graphe de dépendances (`sqlx` / `rsa`).
- **Async Compilation**: L'appel lourd à la compilation `.nhtml` est maintenant exécuté dans un `tokio::task::spawn_blocking` pour éviter de bloquer l'executor asynchrone principal.
- **Binary Hardening**: Pré-allocation mémoire intelligente avec `Vec::with_capacity` pour la construction de l'arbre B-TREE, et troncature stricte des champs à 65535 octets.
- **HashDoS Mitigation**: Migration vers `sha2::Sha256` pour générer les empreintes de logs de sessions (au lieu de `DefaultHasher` qui est prévisible).
- **Strict TLS**: Ajout de la propriété `min_version: "1.3"` dans la configuration TLS.
- **Frontend Thrashing**: Mise en cache drastique du calcul `getBoundingClientRect()` sur le client avec un `ResizeObserver` global, évitant de tuer la batterie sur les événements `mousemove`.
- **Examples Security**: Migration du `counter` vers le SDK Object `Nhtml::patch()`, suppression de l'injection CSS dans `style-lab`, et troncature (`mb_substr`) + `JOIN` (anti N+1) sur l'exemple de `chat`.

## v0.7.0 (Avril 2026) — "Industrial Scale"
### Added
- **Redis Gateway Clustering**: Support natif pour le déploiement multi-nœuds avec synchronisation des sessions via Redis.
- **Filtering Multi-Gateway**: Prévention des boucles de broadcast via filtrage par `gateway_id` unique.
- **Observability Dashboard**: Nouveaux onglets FLOW, METRICS et SESSIONS dans les DevTools.
- **Prometheus Export**: Exportation des métriques système (Clients actifs, Débit paquets) vers Prometheus/Grafana.
- **Structured JSON Logging**: Flag `--json` pour intégration avec ELK et Datadog.
- **Triple-License Model**: MIT (SDKs), AGPLv3 (Gateway Core), et Licence Commerciale.

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
