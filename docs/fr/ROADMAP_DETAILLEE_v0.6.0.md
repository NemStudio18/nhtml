# 🗺️ Roadmap Détaillée NHTML v0.6.0 "Global Connect"

Cette roadmap définit les actions précises à entreprendre pour transformer NHTML en une solution de production ultra-performante et collaborative.

---

## 🚀 Phase 6.1 : Infrastructure Haute Performance (FastCGI)
**Objectif** : Éliminer l'overhead du fork PHP et stabiliser les backends industriels.

### 1. Pool de Connexions Persistantes
- [x] **Action** : Implémenter un système de gestion de pool de sockets dans `src/socket/mod.rs`.
- [x] **Action** : Gérer le `Keep-Alive` pour réutiliser les connexions FastCGI entre plusieurs événements d'une même session.
- [x] **Action** : Ajouter un timeout de socket configurable dans `nhtml.config.toml`.

### 2. Support des Sockets Unix (Linux/macOS)
- [x] **Action** : Étendre le binaire pour accepter les chemins de sockets Unix (ex: `/var/run/php/php8.2-fpm.sock`) en plus de TCP.
- [x] **Action** : Auto-détection du mode (Stream vs Unix) basée sur le préfixe de l'adresse FPM.

---

## 📡 Phase 6.2 : Collaboration en Temps Réel (Broadcasting)
**Objectif** : Permettre des expériences multi-utilisateurs fluides.

### 1. Moteur de Routage Scoped
- [x] **Fait** : Identification de l'expéditeur via `SenderSID`.
- [x] **Action** : Optimiser la boucle de diffusion pour éviter les clones inutiles du payload binaire (utilisation de `Arc<Vec<u8>>`).
- [x] **Action** : Ajouter le support des "Rooms" (groupes de sessions) pour limiter le broadcast à un sous-ensemble d'utilisateurs.

### 2. SDK PHP étendu
- [x] **Action** : Ajouter la méthode `Nhtml::joinRoom()`, `Nhtml::leaveRoom()` et `Nhtml::broadcastInRoom()`.
- [x] **Action** : Permettre l'envoi simultané d'un patch privé ET d'un broadcast public dans la même réponse JSON.

---

## 🛡️ Phase 6.3 : Stabilité & Gestion d'Erreurs
**Objectif** : Zéro crash, 100% de visibilité.

### 1. Refactoring "No-Panic"
- [x] **Fait** : Introduction de `GatewayError`.
- [x] **Action** : Remplacer les derniers `expect()` et `unwrap()` dans `main.rs`, `supervisor.rs`, `socket/mod.rs`, `cli.rs` et `compiler/mod.rs` par une gestion d'erreurs robuste.
- [x] **Action** : Améliorer le logging des erreurs FastCGI (Timeout, Connection Refused) pour les afficher dans le DevTools via `monitor_pkt`.

### 2. Auto-Récupération (Healthchecks)
- [x] **Action** : Le Superviseur tente désormais de redémarrer automatiquement le serveur PHP de développement en cas de crash (boucle d'auto-restart).
- [x] **Action** : Affichage d'alertes visuelles (LOG 0x10) dans le navigateur lorsque le backend PHP/FastCGI est injoignable.

---

## ☁️ Phase 6.4 : Connectivité Cloud & Déploiement
**Objectif** : Faciliter l'accès distant sécurisé et la mise en production.

### 1. Tunneling & CLI
- [ ] **Action** : Ajouter la commande `nhtml share` permettant de créer un tunnel temporaire (via localtunnel ou service tiers) pour présenter un projet.
- [ ] **Action** : Implémenter `nhtml build --production` pour minifier le B-TREE et optimiser les assets statiques.

### 2. Sécurisation Industrielle
- [ ] **Action** : Finaliser le support du TLS natif dans le Gateway (via `rustls`) pour éviter de dépendre d'un reverse-proxy en mode standalone.
- [ ] **Action** : Implémenter un Rate-Limiter par IP pour protéger le backend PHP des attaques par déni de service sur les événements WebSocket.

---

## 💎 Phase 6.5 : Expérience Développeur (DX) Fine
**Objectif** : Rendre NHTML "magique" à l'usage.

### 1. Hot Reload Intelligent
- [ ] **Action** : Améliorer le `watcher` pour ne recharger que les nœuds modifiés (Partial Reload) plutôt que la session entière.
- [ ] **Action** : Intégrer un overlay de debug directement dans la page (mini-dashboard escamotable).

---

> **Status Actuel** : Le socle industriel (FastCGI, Collaboration, No-Panic) est **terminé**. Nous entrons dans la phase de **Connectivité Cloud** et de **Sécurisation**.
