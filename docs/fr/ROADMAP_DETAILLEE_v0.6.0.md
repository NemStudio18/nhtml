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
- [ ] **Action** : Ajouter le support des "Rooms" (groupes de sessions) pour limiter le broadcast à un sous-ensemble d'utilisateurs.

### 2. SDK PHP étendu
- [ ] **Action** : Ajouter la méthode `Nhtml::broadcast($scope)` dans le SDK PHP.
- [ ] **Action** : Permettre l'envoi simultané d'un patch privé ET d'un broadcast public dans la même réponse JSON.

---

## 🛡️ Phase 6.3 : Stabilité & Gestion d'Erreurs
**Objectif** : Zéro crash, 100% de visibilité.

### 1. Refactoring "No-Panic"
- [x] **Fait** : Introduction de `GatewayError`.
- [x] **Action** : Remplacer les derniers `expect()` et `unwrap()` dans `main.rs` et `supervisor.rs` par une remontée d'erreur propre.
- [ ] **Action** : Améliorer le logging des erreurs FastCGI (Timeout, Connection Refused) pour les afficher dans le DevTools.

### 2. Auto-Récupération (Healthchecks)
- [ ] **Action** : Le Superviseur doit tenter de redémarrer le backend s'il détecte un crash systématique.
- [ ] **Action** : Afficher une alerte visuelle dans le navigateur via un paquet `LOG` spécial si le backend est injoignable.

---

## ☁️ Phase 6.4 : Connectivité Cloud & Déploiement
**Objectif** : Faciliter l'accès distant sécurisé.

### 1. Intégration Cloud Tunneling (Optionnel)
- [ ] **Action** : Explorer l'intégration légère d'un binaire `cloudflared` ou `ngrok` pour exposer le Gateway local vers l'extérieur en une commande.
- [ ] **Action** : Ajouter une commande `nhtml tunnel` dans la CLI.

### 2. Documentation de Production
- [ ] **Action** : Créer des fichiers de configuration Nginx et Apache exemplaires incluant la gestion des WebSockets (`Upgrade: websocket`).
- [ ] **Action** : Guide de sécurisation (Fail2Ban, Rate Limiting).

---

> **Status Actuel** : L'architecture de base FastCGI et le Broadcasting sont fonctionnels. La priorité immédiate est la **stabilité du pool** et le **refactoring des erreurs critiques**.
