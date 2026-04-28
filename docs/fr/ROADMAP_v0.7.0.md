# 🗺️ Roadmap NHTML v0.7.0 "Industrial Scale"

Cette roadmap définit les prochaines étapes pour passer à l'échelle supérieure avec NHTML.

---

## ⚡ Phase 7.1 : Clustering & Load Balancing
**Objectif** : Permettre à NHTML de gérer des millions d'utilisateurs simultanés.

### 1. Gateway Cluster Mode
- [x] **Action** : Support de Redis comme backend de synchronisation pour le broadcasting multi-gateway.
- [ ] **Action** : Implémenter un mode "Sticky Sessions" intelligent pour assurer la cohérence des sessions entre plusieurs instances de Gateway.
- [ ] **Action** : Déportation totale de l'état des sessions vers MySQL ou PostgreSQL via un driver agnostique.

### 2. Load Balancing Natif
- [ ] **Action** : Algorithme de Round-Robin ou Least-Connections intégré pour dispatcher vers plusieurs pools FPM.
- [ ] **Action** : Healthchecks avancés des backends avec mise en quarantaine automatique.

---

## 📦 Phase 7.2 : Écosystème Multi-Langages (SDKs)
**Objectif** : Ouvrir NHTML à tous les développeurs, peu importe leur langage backend.

### 1. SDKs Officiels
- [ ] **Action** : Développement du SDK **Python** (FastAPI / Django).
- [ ] **Action** : Développement du SDK **Go** (Gin / Fiber).
- [ ] **Action** : Développement du SDK **Node.js** (Express / NestJS).

### 2. Schema Validation & Types
- [ ] **Action** : Introduction d'un schéma de validation binaire pour les événements afin d'assurer l'intégrité des types entre le client et le serveur.

---

## 🛠️ Phase 7.3 : DevTools & Observabilité
**Objectif** : Offrir une expérience de débogage de niveau professionnel.

### 1. DevTools Avancés
- [ ] **Action** : Time-Travel Debugging : Possibilité de rejouer une séquence de patches binaire pour reproduire un bug.
- [ ] **Action** : Analyseur de performance en temps réel (CPU/Mémoire) par nœud DOM.

### 2. Monitoring & Alerting
- [x] **Action** : Exportation des métriques vers Prometheus / Grafana.
- [x] **Action** : Logs structurés JSON pour intégration facile avec ELK ou Datadog.
