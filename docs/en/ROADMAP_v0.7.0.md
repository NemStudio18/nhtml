# 🗺️ NHTML Roadmap v0.7.0 "Industrial Scale"

This roadmap defines the next steps for scaling NHTML.

---

## ⚡ Phase 7.1: Clustering & Load Balancing
**Goal**: Allow NHTML to handle millions of simultaneous users.

### 1. Gateway Cluster Mode
- [x] **Action**: Support Redis as a synchronization backend for multi-gateway broadcasting.
- [ ] **Action**: Implement intelligent "Sticky Sessions" to ensure session consistency across multiple Gateway instances.
- [ ] **Action**: Full deportation of session state to MySQL or PostgreSQL via an agnostic driver.

### 2. Native Load Balancing
- [ ] **Action**: Integrated Round-Robin or Least-Connections algorithm to dispatch to multiple FPM pools.
- [ ] **Action**: Advanced backend healthchecks with automatic quarantine.

---

## 📦 Phase 7.2: Multi-Language Ecosystem (SDKs)
**Goal**: Open NHTML to all backend developers, regardless of their language.

### 1. Official SDKs
- [ ] **Action**: Development of the **Python** SDK (FastAPI / Django).
- [ ] **Action**: Development of the **Go** SDK (Gin / Fiber).
- [ ] **Action**: Development of the **Node.js** SDK (Express / NestJS).

### 2. Schema Validation & Types
- [ ] **Action**: Introduction of a binary validation schema for events to ensure type integrity between client and server.

---

## 🛠️ Phase 7.3: DevTools & Observability
**Goal**: Provide a professional-grade debugging experience.

### 1. Advanced DevTools
- [ ] **Action**: Time-Travel Debugging: Ability to replay a sequence of binary patches to reproduce a bug.
- [ ] **Action**: Real-time performance analyzer (CPU/Memory) per DOM node.

### 2. Monitoring & Alerting
- [x] **Action**: Export metrics to Prometheus / Grafana.
- [ ] **Action**: Structured JSON logs for easy integration with ELK or Datadog.
