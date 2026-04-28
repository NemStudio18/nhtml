# 🗺️ Detailed Roadmap NHTML v0.6.0 "Global Connect"

This roadmap defines the precise actions required to transform NHTML into an ultra-high-performance, collaborative production solution.

---

## 🚀 Phase 6.1: High-Performance Infrastructure (FastCGI)
**Goal**: Eliminate PHP fork overhead and stabilize industrial backends.

### 1. Persistent Connection Pool
- [x] **Action**: Implement a socket pool management system in `src/socket/mod.rs`.
- [x] **Action**: Manage `Keep-Alive` to reuse FastCGI connections between multiple events in the same session.
- [x] **Action**: Add a configurable socket timeout in `nhtml.config.toml`.

### 2. Unix Socket Support (Linux/macOS)
- [x] **Action**: Extend the binary to accept Unix socket paths (e.g., `/var/run/php/php8.2-fpm.sock`) in addition to TCP.
- [x] **Action**: Auto-detect mode (Stream vs Unix) based on the FPM address prefix.

---

## 📡 Phase 6.2: Real-Time Collaboration (Broadcasting)
**Goal**: Enable seamless multi-user experiences.

### 1. Scoped Routing Engine
- [x] **Done**: Sender identification via `SenderSID`.
- [x] **Action**: Optimize the broadcasting loop to avoid unnecessary binary payload clones (using `Arc<Vec<u8>>`).
- [x] **Action**: Add support for "Rooms" (session groups) to limit broadcasting to a subset of users.

### 2. Extended PHP SDK
- [x] **Action**: Add `Nhtml::joinRoom()`, `Nhtml::leaveRoom()` and `Nhtml::broadcastInRoom()` methods to the PHP SDK.
- [x] **Action**: Allow simultaneous sending of a private patch AND a public broadcast in the same JSON response.

---

## 🛡️ Phase 6.3: Stability & Error Management
**Goal**: Zero crashes, 100% visibility.

### 1. "No-Panic" Refactoring
- [x] **Done**: Introduction of `GatewayError`.
- [x] **Action**: Replace the last `expect()` and `unwrap()` in `main.rs`, `supervisor.rs`, `socket/mod.rs`, `cli.rs` and `compiler/mod.rs` with robust error handling.
- [x] **Action**: Improve logging of FastCGI errors (Timeout, Connection Refused) to display them in DevTools via `monitor_pkt`.

### 2. Auto-Recovery (Healthchecks)
- [x] **Action**: The Supervisor now automatically attempts to restart the development PHP server upon crash (auto-restart loop).
- [x] **Action**: Display visual alerts (LOG 0x10) in the browser when the PHP/FastCGI backend is unreachable.

---

## ☁️ Phase 6.4: Cloud Connectivity & Deployment
**Goal**: Facilitate secure remote access and production readiness.

### 1. Tunneling & CLI
- [ ] **Action**: Add the `nhtml share` command to create a temporary tunnel (via localtunnel or third-party service) for project demos.
- [ ] **Action**: Implement `nhtml build --production` to minify the B-TREE and optimize static assets.

### 2. Industrial Hardening
- [ ] **Action**: Finalize native TLS support in the Gateway (via `rustls`) to avoid dependency on reverse proxies in standalone mode.
- [ ] **Action**: Implement an IP-based Rate-Limiter to protect the PHP backend from DoS attacks via WebSocket events.

---

## 💎 Phase 6.5: Fine-tuned Developer Experience (DX)
**Goal**: Make NHTML "magic" to use.

### 1. Smart Hot Reload
- [ ] **Action**: Improve the `watcher` to only reload modified nodes (Partial Reload) rather than the entire session.
- [ ] **Action**: Integrate a debug overlay directly in the page (retractable mini-dashboard).

---

> **Current Status**: The industrial foundation (FastCGI, Collaboration, No-Panic) is **complete**. We are now moving into the **Cloud Connectivity** and **Hardening** phase.
