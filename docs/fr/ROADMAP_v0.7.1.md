# 🗺️ Roadmap NHTML v0.7.1 "Industrial Hardening"

Cette roadmap définit les prochaines étapes pour passer à l'échelle supérieure avec NHTML.

---

## 🛡️ Phase 7.0 : Stabilisation & Sécurité (Industrial Hardening)
**Objectif** : Renforcer la robustesse du cœur de la Gateway.

### 1. Zero-Panic Policy & Robustesse
- [x] **Action** : Élimination totale des `unwrap()` et `expect()` critiques dans le runtime (Supervisor, CLI, Socket).
- [x] **Action** : Durcissement du **Rate Limiter** (Nettoyage périodique TTL 3600s, seuil dynamique 500 IPs).
- [x] **Action** : Protection contre le **Cross-Site WebSocket Hijacking** via validation de l'en-tête Origin.
- [x] **Action** : Bounds checks rigoureux sur tous les buffers binaires NBPS.

### 2. Performance & Optimisation
- [x] **Action** : Implémentation du **Compilation Cache** via `Arc` (Réutilisation des `CompileResult`).
- [x] **Action** : Migration vers `Arc<HandlerTable>` et pré-sérialisation JSON (Réduction allocations mémoire).
- [x] **Action** : Atomicité des transactions SQLite pour la synchronisation du `last_seq`.
- [x] **Action** : **Industrialisation UI** : Remplacement de 100% des emojis par Font Awesome et branding Premium.

---

## ⚡ Phase 7.1 : Clustering & Load Balancing
**Objectif** : Permettre à NHTML de gérer des millions d'utilisateurs simultanés.

### 1. Gateway Cluster Mode
- [x] **Action** : Support de Redis comme backend de synchronisation pour le broadcasting multi-gateway.
- [x] **Action** : Implémenter un mode "Sticky Sessions" intelligent (Natif via Shared DB v0.7.1).
- [x] **Action** : Déportation totale de l'état des sessions vers SQLite, MySQL ou PostgreSQL via un driver agnostique (sqlx).

### 2. Load Balancing Natif
- [ ] **Action** : Algorithme de Round-Robin ou Least-Connections intégré pour dispatcher vers plusieurs pools FPM.
- [ ] **Action** : Healthchecks avancés des backends avec mise en quarantaine automatique.

---

## 📦 Phase 7.2 : Écosystème Multi-Langages (SDKs)
**Objectif** : Ouvrir NHTML à tous les développeurs, peu importe leur langage backend.

### 1. SDKs Officiels
- [x] **Action** : Développement du SDK **Python** (Support natif FastAPI & Flask).
- [x] **Action** : Développement du SDK **Node.js** (Support natif Express & NestJS).
- [x] **Action** : Développement du SDK **Go** (Support natif Gin / Fiber).
- [x] **Note** : Tous les SDK sont désormais alignés sur NBPS v0.7.1 avec support Scoped Broadcasting.

### 2. Schema Validation & Types
- [x] **Action** : Introduction d'un schéma de validation binaire pour les événements afin d'assurer l'intégrité des types entre le client et le serveur. (Basique implémenté via Gateway + Docs).

---

## 🛠️ Phase 7.3 : DevTools & Observabilité
**Objectif** : Offrir une expérience de débogage de niveau professionnel.

### 1. DevTools Avancés
- [x] **Action** : Time-Travel Debugging : Possibilité de rejouer une séquence de patches binaire pour reproduire un bug. (Implémenté dans CLI + DevTools).
- [ ] **Action** : Analyseur de performance en temps réel (CPU/Mémoire) par nœud DOM.

### 2. Monitoring & Alerting
- [x] **Action** : Exportation des métriques vers Prometheus / Grafana.
- [x] **Action** : Logs structurés JSON pour intégration facile avec ELK ou Datadog.
